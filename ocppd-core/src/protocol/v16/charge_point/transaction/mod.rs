pub(super) mod driver;
pub(super) mod held;
pub(super) mod offline_start;
pub(super) mod retry;

use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::charging_profile::ChargingProfile;
use crate::protocol::v16::messages::datatypes::id_tag::{AuthorizationStatus, IdTagInfo, IdToken};
use crate::protocol::v16::messages::transaction::{
    AuthorizeRequest, AuthorizeResponse, Reason, StartTransactionRequest, StartTransactionResponse,
    StopTransactionRequest,
};
use crate::wire::date_time::DateTime;
use crate::wire::outbox::Call;
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Idle,
    Authorizing {
        id_tag: IdToken,
        profile: Option<ChargingProfile>,
    },
    Starting {
        id_tag: IdToken,
        profile: Option<ChargingProfile>,
    },
    Active {
        transaction_id: Option<i32>,
        id_tag: IdToken,
        start: u64,
        last_sample: u64,
        last_register_wh: f64,
    },
    Finished,
}

impl ChargePoint {
    pub(crate) fn transaction_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        call: &Call,
        payload: Value,
    ) {
        let index = call.connector_id as usize - 1;
        match (call.action.as_str(), self.transactions[index].clone()) {
            ("Authorize", Transaction::Authorizing { id_tag, profile }) => {
                if let Some(response) =
                    self.decode::<AuthorizeResponse>(out, now, "Authorize", payload)
                {
                    self.remember(&id_tag, &response.id_tag_info);
                    self.authorized(out, now, index, id_tag, response.id_tag_info, profile);
                } else {
                    self.transactions[index] = Transaction::Idle;
                }
            }
            ("StartTransaction", Transaction::Starting { id_tag, profile }) => {
                if let Some(response) =
                    self.decode::<StartTransactionResponse>(out, now, "StartTransaction", payload)
                {
                    if let Some(profile) = profile {
                        self.install_transaction_profile(
                            out,
                            now,
                            index,
                            response.transaction_id,
                            profile,
                        );
                    }
                    self.started(out, now, index, id_tag, response);
                } else {
                    self.transactions[index] = Transaction::Idle;
                }
            }
            (
                "StartTransaction",
                Transaction::Active {
                    transaction_id: None,
                    ..
                },
            ) => {
                if let Some(response) =
                    self.decode::<StartTransactionResponse>(out, now, "StartTransaction", payload)
                {
                    self.started_offline(out, now, index, response);
                }
            }
            ("StartTransaction", _) => {
                if let Some(response) =
                    self.decode::<StartTransactionResponse>(out, now, "StartTransaction", payload)
                {
                    let id = response.transaction_id;
                    if !self.fill_transaction_id(index, id) {
                        self.stop_orphan(now, index, id);
                    }
                }
            }
            _ => {}
        }
    }

    fn stop_orphan(&mut self, now: u64, index: usize, transaction_id: i32) {
        let request = StopTransactionRequest {
            id_tag: None,
            meter_stop: self.station.connectors[index].register_wh.round() as i32,
            timestamp: DateTime { unix_ms: now },
            transaction_id,
            reason: Some(Reason::Other),
            transaction_data: None,
        };
        self.outbox.push("StopTransaction", 0, json!(request));
    }

    pub(crate) fn transaction_failed(&mut self, call: &Call) {
        let index = call.connector_id as usize - 1;
        match self.transactions[index] {
            Transaction::Authorizing { .. } | Transaction::Starting { .. } => {
                self.transactions[index] = Transaction::Idle;
            }
            _ => {}
        }
    }

    pub(crate) fn connection_lost(&mut self) {
        self.resend_statuses = true;
        if self.held_deadline().is_none() {
            self.held = None;
        }
        for transaction in &mut self.transactions {
            if let Transaction::Authorizing { .. } = transaction {
                *transaction = Transaction::Idle;
            }
        }
    }

    pub(crate) fn authorize_on(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_tag: IdToken,
        profile: Option<ChargingProfile>,
    ) {
        if let Some(info) = self.locally_authorized(&id_tag, now) {
            return self.authorized(out, now, index, id_tag, info, profile);
        }
        let request = AuthorizeRequest {
            id_tag: id_tag.clone(),
        };
        self.outbox
            .push("Authorize", index as u32 + 1, json!(request));
        self.transactions[index] = Transaction::Authorizing { id_tag, profile };
    }

    fn authorized(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_tag: IdToken,
        info: IdTagInfo,
        profile: Option<ChargingProfile>,
    ) {
        let connector = index as u32 + 1;
        let status = info.status;
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
        if self.reserved_for_other(index, &id_tag, info.parent_id_tag.as_ref()) {
            self.transactions[index] = Transaction::Idle;
            return self.reject(out, now, Some(connector), "reserved");
        }
        self.start_transaction(out, now, index, id_tag);
        self.attach_profile(index, profile);
    }

    pub(crate) fn start_transaction(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_tag: IdToken,
    ) {
        let connector = index as u32 + 1;
        let reservation_id = self.use_reservation(out, now, index);
        let request = StartTransactionRequest {
            connector_id: connector,
            id_tag: id_tag.clone(),
            meter_start: self.station.connectors[index].register_wh.round() as i32,
            reservation_id,
            timestamp: DateTime { unix_ms: now },
        };
        self.outbox
            .push("StartTransaction", connector, json!(request));
        match self.is_connected() {
            true => {
                self.transactions[index] = Transaction::Starting {
                    id_tag,
                    profile: None,
                }
            }
            false => self.begin_charging(now, index, None, id_tag),
        }
    }

    fn started(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_tag: IdToken,
        response: StartTransactionResponse,
    ) {
        self.remember(&id_tag, &response.id_tag_info);
        let connector = index as u32 + 1;
        let transaction_id = response.transaction_id;
        let kind = EventKind::TransactionStarted {
            connector,
            transaction_id,
        };
        self.emit(out, now, kind);
        self.begin_charging(now, index, Some(transaction_id), id_tag);
        if response.id_tag_info.status != AuthorizationStatus::Accepted {
            return self.stop(out, now, index, Reason::DeAuthorized);
        }
        if self.station.connectors[index].plugged.is_none() {
            self.stop(out, now, index, Reason::EvDisconnected);
        }
    }

    fn begin_charging(
        &mut self,
        now: u64,
        index: usize,
        transaction_id: Option<i32>,
        id_tag: IdToken,
    ) {
        self.transactions[index] = Transaction::Active {
            transaction_id,
            id_tag,
            start: now,
            last_sample: now,
            last_register_wh: self.station.connectors[index].register_wh,
        };
        let limit_w = self.composite_limit_w(index as u32 + 1, now);
        let supply = self.station.supply(index);
        self.station.connectors[index].start_charging(now, supply, limit_w);
    }

    pub(crate) fn stop(&mut self, out: &mut Vec<Output>, now: u64, index: usize, reason: Reason) {
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
            transaction_id: transaction_id.unwrap_or_default(),
            reason: Some(reason),
            transaction_data: None,
        };
        let mut payload = json!(request);
        if let (None, Some(fields)) = (transaction_id, payload.as_object_mut()) {
            fields.remove("transactionId");
        }
        self.outbox
            .push("StopTransaction", index as u32 + 1, payload);
        let kind = EventKind::TransactionStopped {
            connector: index as u32 + 1,
            transaction_id,
            reason,
        };
        self.emit(out, now, kind);
        self.transaction_ended(index);
    }

    fn transaction_ended(&mut self, index: usize) {
        self.charging_profiles.end_transaction(index as u32 + 1);
        let plugged = self.station.connectors[index].plugged.is_some();
        self.transactions[index] = if plugged {
            Transaction::Finished
        } else {
            Transaction::Idle
        };
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
