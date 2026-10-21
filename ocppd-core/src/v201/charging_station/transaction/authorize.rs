use super::Authorization;
use crate::api::io::Output;
use crate::v201::charging_station::ChargingStation;
use crate::v201::event::EventKind;
use crate::v201::messages::authorization::{AuthorizeRequest, AuthorizeResponse};
use crate::v201::messages::datatypes::id_token::{AuthorizationStatus, IdToken, IdTokenType};
use crate::v201::messages::transactions::TriggerReason;
use crate::wire::ci_string::CiString;
use crate::wire::outbox::Call;
use serde_json::{Value, json};

impl ChargingStation {
    pub(in crate::v201::charging_station) fn present_id_token(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        evse: Option<u32>,
        id_tag: &str,
    ) {
        let Some(index) = self.evse_for_id_token(evse) else {
            return self.reject(out, now, evse, "no connector to authorize");
        };
        let Ok(id_token) = CiString::new(id_tag) else {
            return self.reject(out, now, evse, "id tag longer than 36 characters");
        };
        let id_token = IdToken {
            additional_info: None,
            kind: token_type(id_tag),
            id_token,
        };
        let evse = Some(index as u32 + 1);
        match &self.sessions[index].authorization {
            Authorization::Authorized {
                id_token: started, ..
            } if started.id_token.matches(&id_token.id_token) => {
                self.sessions[index].authorization = Authorization::None;
                self.settle(out, now, index, TriggerReason::StopAuthorized);
            }
            Authorization::Authorized { .. } => {
                self.reject(out, now, evse, "another id tag is authorized")
            }
            Authorization::Pending(_) => self.reject(out, now, evse, "an authorization is pending"),
            Authorization::None => {
                let request = AuthorizeRequest {
                    id_token: id_token.clone(),
                    certificate: None,
                    iso15118_certificate_hash_data: None,
                };
                self.outbox
                    .push("Authorize", index as u32 + 1, json!(request));
                self.sessions[index].authorization = Authorization::Pending(id_token);
            }
        }
    }

    pub(in crate::v201::charging_station) fn authorize_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        call: &Call,
        payload: Value,
    ) {
        let index = call.connector_id as usize - 1;
        let Authorization::Pending(id_token) = self.sessions[index].authorization.clone() else {
            return;
        };
        let status = self
            .decode::<AuthorizeResponse>(out, now, "Authorize", payload)
            .map(|response| response.id_token_info.status);
        let kind = EventKind::Authorization {
            evse: call.connector_id,
            id_tag: id_token.id_token.as_str().to_owned(),
            status,
        };
        self.emit(out, now, kind);
        self.sessions[index].authorization = match status {
            Some(AuthorizationStatus::Accepted) => Authorization::Authorized { id_token, at: now },
            _ => Authorization::None,
        };
        self.settle(out, now, index, TriggerReason::Authorized);
    }

    pub(in crate::v201::charging_station) fn authorize_failed(&mut self, call: &Call) {
        let index = call.connector_id as usize - 1;
        if let Authorization::Pending(_) = self.sessions[index].authorization {
            self.sessions[index].authorization = Authorization::None;
        }
    }

    fn evse_for_id_token(&self, evse: Option<u32>) -> Option<usize> {
        let count = self.station.connectors.len();
        match evse {
            Some(evse) => evse
                .checked_sub(1)
                .map(|index| index as usize)
                .filter(|&index| index < count),
            None if count == 1 => Some(0),
            None => (0..count).find(|&index| {
                self.station.connectors[index].plugged.is_some()
                    && self.sessions[index].authorization == Authorization::None
            }),
        }
    }
}

fn token_type(id_tag: &str) -> IdTokenType {
    let hex = id_tag.chars().all(|c| c.is_ascii_hexdigit());
    match id_tag.len() {
        8 | 14 if hex => IdTokenType::Iso14443,
        16 if hex => IdTokenType::Iso15693,
        _ => IdTokenType::Local,
    }
}

#[cfg(test)]
#[path = "authorize.test.rs"]
mod tests;
