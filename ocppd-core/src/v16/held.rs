use super::charge_point::ChargePoint;
use super::configuration::CONNECTION_TIME_OUT;
use super::messages::id_tag::{AuthorizationStatus, IdToken};
use super::messages::transaction::{AuthorizeRequest, AuthorizeResponse};
use super::transaction::Transaction;
use crate::event::{EventKind, Output};
use serde_json::{Value, json};

pub struct Held {
    id_tag: IdToken,
    connector: Option<usize>,
    expires: Option<u64>,
}

impl ChargePoint {
    pub(super) fn hold_for_authorization(&mut self, id_tag: IdToken, connector: Option<usize>) {
        let request = AuthorizeRequest {
            id_tag: id_tag.clone(),
        };
        self.outbox.push("Authorize", 0, json!(request));
        self.held = Some(Held {
            id_tag,
            connector,
            expires: None,
        });
    }

    pub(super) fn hold_authorized(&mut self, now: u64, id_tag: IdToken, connector: Option<usize>) {
        self.held = Some(Held {
            id_tag,
            connector,
            expires: Some(now + self.connection_time_out()),
        });
    }

    pub(super) fn held_reply(&mut self, out: &mut Vec<Output>, now: u64, payload: Value) {
        let Some(held) = self.held.take() else {
            return;
        };
        let Some(response) = self.decode::<AuthorizeResponse>(out, now, "Authorize", payload)
        else {
            return;
        };
        let status = response.id_tag_info.status;
        let kind = EventKind::Authorization {
            connector: held.connector.map(|index| index as u32 + 1),
            id_tag: held.id_tag.as_str().to_owned(),
            status,
        };
        self.emit(out, now, kind);
        if status == AuthorizationStatus::Accepted {
            self.hold_authorized(now, held.id_tag, held.connector);
            self.start_held(now);
        }
    }

    pub(super) fn start_held(&mut self, now: u64) {
        let Some(Held {
            connector,
            expires: Some(_),
            ..
        }) = self.held
        else {
            return;
        };
        let ready = |index: usize| {
            self.station.connectors[index].plugged.is_some()
                && self.transactions[index] == Transaction::Idle
                && !self.unavailable(index)
        };
        let index = match connector {
            Some(index) => Some(index).filter(|&index| ready(index)),
            None => (0..self.transactions.len()).find(|&index| ready(index)),
        };
        let Some(index) = index else {
            return;
        };
        if let Some(held) = self.held.take() {
            self.start_transaction(now, index, held.id_tag);
        }
    }

    pub(super) fn expire_held(&mut self, out: &mut Vec<Output>, now: u64) {
        let expired =
            matches!(self.held, Some(Held { expires: Some(expires), .. }) if now >= expires);
        if !expired {
            return;
        }
        if let Some(held) = self.held.take() {
            let id_tag = held.id_tag.as_str().to_owned();
            self.emit(out, now, EventKind::AuthorizationExpired { id_tag });
        }
    }

    pub(super) fn held_deadline(&self) -> Option<u64> {
        self.held.as_ref()?.expires
    }

    pub(super) fn held_for(&self, index: usize) -> bool {
        matches!(&self.held, Some(held) if held.connector == Some(index))
    }

    fn connection_time_out(&self) -> u64 {
        self.configuration
            .interval_ms(CONNECTION_TIME_OUT)
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "held.test.rs"]
mod tests;
