use super::ChargePoint;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::core::transaction::{
    AuthorizeResponse, Reason, StartTransactionRequest, StartTransactionResponse,
    StopTransactionRequest,
};
use crate::v16::messages::datatypes::id_tag::{AuthorizationStatus, IdToken};
use crate::wire::date_time::DateTime;
use crate::wire::outbox::Call;
use serde_json::{Value, json};

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
        last_sample: u64,
        last_register_wh: f64,
    },
    Stopping,
    Finished,
}

impl ChargePoint {
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
        self.resend_statuses = true;
        if self.held_deadline().is_none() {
            self.held = None;
        }
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
            connector: Some(connector),
            id_tag: String::from(id_tag.clone()),
            status,
        };
        self.emit(out, now, kind);
        let plugged = self.station.connectors[index].plugged.is_some();
        if status != AuthorizationStatus::Accepted || !plugged {
            self.transactions[index] = Transaction::Idle;
            return;
        }
        self.start_transaction(now, index, id_tag);
    }

    pub(super) fn start_transaction(&mut self, now: u64, index: usize, id_tag: IdToken) {
        let connector = index as u32 + 1;
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
            last_sample: now,
            last_register_wh: self.station.connectors[index].register_wh,
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

    pub(super) fn stop(&mut self, out: &mut Vec<Output>, now: u64, index: usize, reason: Reason) {
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
