use super::charge_point::ChargePoint;
use super::messages::id_tag::{AuthorizationStatus, IdToken};
use super::messages::transaction::{
    AuthorizeRequest, AuthorizeResponse, Reason, StartTransactionRequest, StartTransactionResponse,
    StopTransactionRequest,
};
use crate::ci_string::CiString;
use crate::date_time::DateTime;
use crate::event::{EventKind, Output};
use crate::intent::{EvSpec, Intent};
use crate::outbox::Call;
use serde_json::{Value, json};

pub(super) const METER_VALUE_SAMPLE_INTERVAL_MS: u64 = 60_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Idle,
    Authorizing {
        id_tag: IdToken,
    },
    Starting {
        id_tag: IdToken,
    },
    Active {
        transaction_id: i32,
        id_tag: IdToken,
        next_sample: u64,
    },
    Stopping,
    Finished,
}

impl ChargePoint {
    pub(super) fn intent(&mut self, out: &mut Vec<Output>, now: u64, intent: Intent) {
        let connector = intent.connector();
        let Some(index) = connector
            .checked_sub(1)
            .map(|index| index as usize)
            .filter(|&index| index < self.transactions.len())
        else {
            return self.reject(out, now, connector, "unknown connector");
        };
        match intent {
            Intent::PlugIn { ev, .. } => self.plug_in(out, now, index, ev),
            Intent::Authorize { id_tag, .. } => self.authorize(out, now, index, &id_tag),
            Intent::Stop { .. } => self.stop_by_driver(out, now, index),
            Intent::Unplug { .. } => self.unplug(out, now, index),
        }
    }

    pub(super) fn transaction_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        call: &Call,
        payload: Value,
    ) {
        let index = call.connector_id as usize - 1;
        match (call.action.as_str(), self.transactions[index].clone()) {
            ("Authorize", Transaction::Authorizing { id_tag }) => {
                if let Some(response) =
                    self.decode::<AuthorizeResponse>(out, now, "Authorize", payload)
                {
                    self.authorized(out, now, index, id_tag, response.id_tag_info.status);
                } else {
                    self.transactions[index] = Transaction::Idle;
                }
            }
            ("StartTransaction", Transaction::Starting { id_tag }) => {
                if let Some(response) =
                    self.decode::<StartTransactionResponse>(out, now, "StartTransaction", payload)
                {
                    self.started(out, now, index, id_tag, response);
                } else {
                    self.transactions[index] = Transaction::Idle;
                }
            }
            ("StopTransaction", Transaction::Stopping) => self.transaction_ended(index),
            _ => {}
        }
    }

    pub(super) fn transaction_failed(&mut self, call: &Call) {
        let index = call.connector_id as usize - 1;
        match self.transactions[index] {
            Transaction::Authorizing { .. } | Transaction::Starting { .. } => {
                self.transactions[index] = Transaction::Idle;
            }
            Transaction::Stopping => self.transaction_ended(index),
            _ => {}
        }
    }

    pub(super) fn connection_lost(&mut self) {
        self.reported.fill(None);
        for index in 0..self.transactions.len() {
            match self.transactions[index] {
                Transaction::Authorizing { .. } | Transaction::Starting { .. } => {
                    self.transactions[index] = Transaction::Idle;
                }
                Transaction::Stopping => self.transaction_ended(index),
                _ => {}
            }
        }
    }

    fn plug_in(&mut self, out: &mut Vec<Output>, now: u64, index: usize, ev: EvSpec) {
        let connector = &mut self.station.connectors[index];
        if connector.plugged.is_some() {
            return self.reject(out, now, index as u32 + 1, "already plugged in");
        }
        connector.plug_in(ev.ev(), ev.soc);
    }

    fn authorize(&mut self, out: &mut Vec<Output>, now: u64, index: usize, id_tag: &str) {
        let connector = index as u32 + 1;
        if self.station.connectors[index].plugged.is_none() {
            return self.reject(out, now, connector, "nothing plugged in");
        }
        if self.transactions[index] != Transaction::Idle {
            return self.reject(out, now, connector, "transaction in progress");
        }
        let id_tag = match CiString::new(id_tag) {
            Ok(id_tag) => id_tag,
            Err(err) => return self.reject(out, now, connector, &err.to_string()),
        };
        let request = AuthorizeRequest {
            id_tag: id_tag.clone(),
        };
        self.outbox.push("Authorize", connector, json!(request));
        self.transactions[index] = Transaction::Authorizing { id_tag };
    }

    fn authorized(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_tag: IdToken,
        status: AuthorizationStatus,
    ) {
        let connector = index as u32 + 1;
        let kind = EventKind::Authorization {
            connector,
            id_tag: String::from(id_tag.clone()),
            status,
        };
        self.emit(out, now, kind);
        let plugged = self.station.connectors[index].plugged.is_some();
        if status != AuthorizationStatus::Accepted || !plugged {
            self.transactions[index] = Transaction::Idle;
            return;
        }
        let request = StartTransactionRequest {
            connector_id: connector,
            id_tag: id_tag.clone(),
            meter_start: self.station.connectors[index].register_wh.round() as i32,
            reservation_id: None,
            timestamp: DateTime { unix_ms: now },
        };
        self.outbox
            .push("StartTransaction", connector, json!(request));
        self.transactions[index] = Transaction::Starting { id_tag };
    }

    fn started(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_tag: IdToken,
        response: StartTransactionResponse,
    ) {
        let connector = index as u32 + 1;
        let transaction_id = response.transaction_id;
        let kind = EventKind::TransactionStarted {
            connector,
            transaction_id,
        };
        self.emit(out, now, kind);
        self.transactions[index] = Transaction::Active {
            transaction_id,
            id_tag,
            next_sample: now + METER_VALUE_SAMPLE_INTERVAL_MS,
        };
        if response.id_tag_info.status != AuthorizationStatus::Accepted {
            return self.stop(out, now, index, Reason::DeAuthorized);
        }
        if self.station.connectors[index].plugged.is_none() {
            return self.stop(out, now, index, Reason::EvDisconnected);
        }
        let limit_w = self.station.limit_w();
        self.station.connectors[index].start_charging(now, limit_w);
    }

    fn stop_by_driver(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        if !matches!(self.transactions[index], Transaction::Active { .. }) {
            return self.reject(out, now, index as u32 + 1, "no active transaction");
        }
        self.stop(out, now, index, Reason::Local);
    }

    fn unplug(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        if self.station.connectors[index].plugged.is_none() {
            return self.reject(out, now, index as u32 + 1, "nothing plugged in");
        }
        match self.transactions[index] {
            Transaction::Active { .. } => self.stop(out, now, index, Reason::EvDisconnected),
            Transaction::Finished => self.transactions[index] = Transaction::Idle,
            _ => {}
        }
        self.station.connectors[index].unplug(now);
    }

    fn stop(&mut self, out: &mut Vec<Output>, now: u64, index: usize, reason: Reason) {
        let Transaction::Active {
            transaction_id,
            id_tag,
            ..
        } = self.transactions[index].clone()
        else {
            return;
        };
        let connector = &mut self.station.connectors[index];
        connector.stop_charging(now);
        let request = StopTransactionRequest {
            id_tag: Some(id_tag),
            meter_stop: connector.register_wh.round() as i32,
            timestamp: DateTime { unix_ms: now },
            transaction_id,
            reason: Some(reason),
            transaction_data: None,
        };
        self.outbox
            .push("StopTransaction", index as u32 + 1, json!(request));
        let kind = EventKind::TransactionStopped {
            connector: index as u32 + 1,
            transaction_id,
            reason,
        };
        self.emit(out, now, kind);
        self.transactions[index] = Transaction::Stopping;
    }

    fn transaction_ended(&mut self, index: usize) {
        let plugged = self.station.connectors[index].plugged.is_some();
        self.transactions[index] = if plugged {
            Transaction::Finished
        } else {
            Transaction::Idle
        };
    }
}

#[cfg(test)]
#[path = "transaction.test.rs"]
mod tests;
