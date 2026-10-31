use super::Transaction;
use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::provisioning::configuration::CONNECTION_TIME_OUT;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::charging_profile::ChargingProfile;
use crate::protocol::v16::messages::datatypes::id_tag::{AuthorizationStatus, IdToken};
use crate::protocol::v16::messages::transaction::{AuthorizeRequest, AuthorizeResponse};
use serde_json::{Value, json};

pub struct Held {
    id_tag: IdToken,
    connector: Option<usize>,
    expires: Option<u64>,
    profile: Option<ChargingProfile>,
}

impl ChargePoint {
    pub(crate) fn hold_for_authorization(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        id_tag: IdToken,
        connector: Option<usize>,
        profile: Option<ChargingProfile>,
    ) {
        let held = Held {
            id_tag,
            connector,
            expires: None,
            profile,
        };
        if self.locally_authorized(&held.id_tag, now).is_some() {
            return self.held_decided(out, now, held, AuthorizationStatus::Accepted);
        }
        let request = AuthorizeRequest {
            id_tag: held.id_tag.clone(),
        };
        self.outbox.push("Authorize", 0, json!(request));
        self.held = Some(held);
    }

    pub(crate) fn hold_authorized(
        &mut self,
        now: u64,
        id_tag: IdToken,
        connector: Option<usize>,
        profile: Option<ChargingProfile>,
    ) {
        self.held = Some(Held {
            id_tag,
            connector,
            expires: Some(now + self.connection_time_out()),
            profile,
        });
    }

    pub(crate) fn held_reply(&mut self, out: &mut Vec<Output>, now: u64, payload: Value) {
        let Some(held) = self.held.take() else {
            return;
        };
        let Some(response) = self.decode::<AuthorizeResponse>(out, now, "Authorize", payload)
        else {
            return;
        };
        self.remember(&held.id_tag, &response.id_tag_info);
        let status = response.id_tag_info.status;
        self.held_decided(out, now, held, status);
    }

    fn held_decided(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        held: Held,
        status: AuthorizationStatus,
    ) {
        let kind = EventKind::Authorization {
            connector: held.connector.map(|index| index as u32 + 1),
            id_tag: held.id_tag.as_str().to_owned(),
            status,
        };
        self.emit(out, now, kind);
        if status == AuthorizationStatus::Accepted {
            self.hold_authorized(now, held.id_tag, held.connector, held.profile);
            self.start_held(out, now);
        }
    }

    pub(crate) fn start_held(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(Held {
            id_tag,
            connector,
            expires: Some(_),
            ..
        }) = &self.held
        else {
            return;
        };
        let ready = |index: usize| {
            self.station.connectors[index].plugged.is_some()
                && self.transactions[index] == Transaction::Idle
                && !self.unavailable(index)
                && !self.reserved_for_other(index, id_tag, None)
        };
        let index = match connector {
            Some(index) => Some(*index).filter(|&index| ready(index)),
            None => (0..self.transactions.len()).find(|&index| ready(index)),
        };
        let Some(index) = index else {
            return;
        };
        if let Some(held) = self.held.take() {
            self.start_transaction(out, now, index, held.id_tag);
            self.attach_profile(index, held.profile);
        }
    }

    pub(crate) fn expire_held(&mut self, out: &mut Vec<Output>, now: u64) {
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

    pub(crate) fn held_deadline(&self) -> Option<u64> {
        self.held.as_ref()?.expires
    }

    pub(crate) fn held_for(&self, index: usize) -> bool {
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
