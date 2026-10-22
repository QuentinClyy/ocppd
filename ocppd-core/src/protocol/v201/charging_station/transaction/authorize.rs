use super::Authorization;
use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::table::{
    AUTH_CTRLR, MASTER_PASS_GROUP_ID,
};
use crate::protocol::v201::charging_station::local_auth::Decision;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::authorization::{AuthorizeRequest, AuthorizeResponse};
use crate::protocol::v201::messages::datatypes::id_token::{
    AuthorizationStatus, IdToken, IdTokenType,
};
use crate::protocol::v201::messages::transactions::{StoppedReason, TriggerReason};
use crate::wire::ci_string::CiString;
use crate::wire::outbox::Call;
use serde_json::{Value, json};

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn present_id_token(
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
        if !self.operative(index) {
            return self.reject(out, now, evse, "connector unavailable");
        }
        match self.sessions[index].authorization.clone() {
            Authorization::Authorized {
                id_token: started, ..
            } if started.id_token.matches(&id_token.id_token) => {
                self.sessions[index].authorization = Authorization::None;
                self.settle(out, now, index, TriggerReason::StopAuthorized);
            }
            Authorization::Pending(_) => self.reject(out, now, evse, "an authorization is pending"),
            Authorization::Authorized { .. } if self.sessions[index].checking.is_some() => {
                self.reject(out, now, evse, "an authorization is pending")
            }
            Authorization::Authorized { group: started, .. } => match self.decide(now, &id_token) {
                Decision::Local(group) => {
                    if self.is_master_pass(group.as_ref()) {
                        return self.stop_all(out, now);
                    }
                    let same = matches!((&group, &started), (Some(group), Some(started)) if group.matches(started));
                    if same {
                        self.sessions[index].authorization = Authorization::None;
                        return self.settle(out, now, index, TriggerReason::StopAuthorized);
                    }
                    self.ask(out, now, index, id_token, false);
                }
                Decision::Ask => self.ask(out, now, index, id_token, false),
                Decision::Refuse => self.reject(out, now, evse, "id not authorized"),
            },
            Authorization::None => match self.decide(now, &id_token) {
                Decision::Local(group) => {
                    let kind = EventKind::Authorization {
                        evse: index as u32 + 1,
                        id_tag: id_token.id_token.as_str().to_owned(),
                        status: Some(AuthorizationStatus::Accepted),
                    };
                    self.emit(out, now, kind);
                    if self.is_master_pass(group.as_ref()) {
                        return self.stop_all(out, now);
                    }
                    self.sessions[index].authorization = Authorization::Authorized {
                        id_token,
                        group,
                        at: now,
                    };
                    self.settle(out, now, index, TriggerReason::Authorized);
                }
                Decision::Ask => self.ask(out, now, index, id_token, true),
                Decision::Refuse => self.reject(out, now, evse, "id not authorized"),
            },
        }
    }

    fn ask(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_token: IdToken,
        starting: bool,
    ) {
        if !self.is_accepted() {
            let evse = Some(index as u32 + 1);
            return self.reject(out, now, evse, "offline: the id cannot be authorized");
        }
        match starting {
            true => self.sessions[index].authorization = Authorization::Pending(id_token.clone()),
            false => self.sessions[index].checking = Some(id_token.clone()),
        }
        self.send_authorize(index, id_token);
    }

    pub(in crate::protocol::v201::charging_station) fn authorize_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        call: &Call,
        payload: Value,
    ) {
        let index = call.connector_id as usize - 1;
        let session = &mut self.sessions[index];
        let (id_token, starting) = match session.authorization.clone() {
            Authorization::Pending(id_token) => (id_token, true),
            _ => match session.checking.take() {
                Some(id_token) => (id_token, false),
                None => return,
            },
        };
        let info = self
            .decode::<AuthorizeResponse>(out, now, "Authorize", payload)
            .map(|response| response.id_token_info);
        if let Some(info) = &info {
            self.remember(&id_token, info.clone(), now);
        }
        let status = info.as_ref().map(|info| info.status);
        let group = info
            .and_then(|info| info.group_id_token)
            .map(|group| group.id_token);
        let kind = EventKind::Authorization {
            evse: call.connector_id,
            id_tag: id_token.id_token.as_str().to_owned(),
            status,
        };
        self.emit(out, now, kind);
        let accepted = status == Some(AuthorizationStatus::Accepted);
        if accepted && self.is_master_pass(group.as_ref()) {
            if starting {
                self.sessions[index].authorization = Authorization::None;
            }
            return self.stop_all(out, now);
        }
        if starting {
            self.sessions[index].authorization = match accepted {
                true => Authorization::Authorized {
                    id_token,
                    group,
                    at: now,
                },
                false => Authorization::None,
            };
            let trigger = match self.sessions[index].remote_start_id {
                Some(_) => TriggerReason::RemoteStart,
                None => TriggerReason::Authorized,
            };
            return self.settle(out, now, index, trigger);
        }
        let same_group = match &self.sessions[index].authorization {
            Authorization::Authorized {
                group: Some(started),
                ..
            } => group.as_ref().is_some_and(|group| group.matches(started)),
            _ => false,
        };
        if accepted && same_group {
            self.sessions[index].authorization = Authorization::None;
            self.settle(out, now, index, TriggerReason::StopAuthorized);
        }
    }

    pub(in crate::protocol::v201::charging_station) fn authorize_failed(&mut self, call: &Call) {
        let session = &mut self.sessions[call.connector_id as usize - 1];
        if let Authorization::Pending(_) = session.authorization {
            session.authorization = Authorization::None;
        }
        session.checking = None;
    }

    pub(in crate::protocol::v201::charging_station) fn authorize_remote(
        &mut self,
        index: usize,
        id_token: IdToken,
    ) {
        self.sessions[index].authorization = Authorization::Pending(id_token.clone());
        self.send_authorize(index, id_token);
    }

    fn send_authorize(&mut self, index: usize, id_token: IdToken) {
        let request = AuthorizeRequest {
            id_token,
            certificate: None,
            iso15118_certificate_hash_data: None,
        };
        self.outbox
            .push("Authorize", index as u32 + 1, json!(request));
    }

    fn is_master_pass(&self, group: Option<&CiString<36>>) -> bool {
        let master = self.device_model.value(AUTH_CTRLR, MASTER_PASS_GROUP_ID);
        let master = master.filter(|master| !master.is_empty());
        match (master, group) {
            (Some(master), Some(group)) => group.as_str().eq_ignore_ascii_case(master),
            _ => false,
        }
    }

    fn stop_all(&mut self, out: &mut Vec<Output>, now: u64) {
        for index in 0..self.sessions.len() {
            let trigger = TriggerReason::StopAuthorized;
            self.end_transaction(out, now, index, trigger, StoppedReason::MasterPass);
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
