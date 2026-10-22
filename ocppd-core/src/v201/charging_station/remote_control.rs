use super::ChargingStation;
use super::device_model::table::AUTH_CTRLR;
use super::transaction::Authorization;
use crate::api::io::Output;
use crate::v201::event::EventKind;
use crate::v201::messages::datatypes::id_token::IdTokenType;
use crate::v201::messages::remote_control::{
    RequestStartStopStatus, RequestStartTransactionRequest, RequestStartTransactionResponse,
    RequestStopTransactionRequest, RequestStopTransactionResponse, UnlockConnectorRequest,
    UnlockConnectorResponse, UnlockStatus,
};
use crate::v201::messages::transactions::TriggerReason;

impl ChargingStation {
    pub(super) fn request_start(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: RequestStartTransactionRequest,
    ) -> RequestStartTransactionResponse {
        let rejected = RequestStartTransactionResponse {
            status: RequestStartStopStatus::Rejected,
            status_info: None,
            transaction_id: None,
        };
        let Some(index) = self.evse_for_remote_start(request.evse_id) else {
            return rejected;
        };
        if !self.is_accepted() || !self.operative(index) {
            return rejected;
        }
        let id_token = request.id_token;
        let check_first = self.device_model.value(AUTH_CTRLR, "AuthorizeRemoteStart")
            == Some("true")
            && !matches!(
                id_token.kind,
                IdTokenType::Central | IdTokenType::NoAuthorization
            );
        let kind = EventKind::RemoteStart {
            evse: index as u32 + 1,
            id_tag: id_token.id_token.as_str().to_owned(),
            remote_start_id: request.remote_start_id,
        };
        self.emit(out, now, kind);
        let session = &mut self.sessions[index];
        session.remote_start_id = Some(request.remote_start_id);
        let transaction_id = session
            .transaction
            .as_ref()
            .map(|transaction| transaction.id.clone());
        match check_first {
            true => self.authorize_remote(index, id_token),
            false => {
                session.authorization = Authorization::Authorized {
                    id_token,
                    group: request.group_id_token.map(|group| group.id_token),
                    at: now,
                };
                self.settle(out, now, index, TriggerReason::RemoteStart);
            }
        }
        RequestStartTransactionResponse {
            status: RequestStartStopStatus::Accepted,
            status_info: None,
            transaction_id,
        }
    }

    pub(super) fn request_stop(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: RequestStopTransactionRequest,
    ) -> RequestStopTransactionResponse {
        let found = self.sessions.iter().position(|session| {
            session
                .transaction
                .as_ref()
                .is_some_and(|transaction| transaction.id.matches(&request.transaction_id))
        });
        let status = match found {
            Some(index) => {
                self.sessions[index].authorization = Authorization::None;
                self.settle(out, now, index, TriggerReason::RemoteStop);
                RequestStartStopStatus::Accepted
            }
            None => RequestStartStopStatus::Rejected,
        };
        RequestStopTransactionResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn unlock_connector(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: UnlockConnectorRequest,
    ) -> UnlockConnectorResponse {
        let index = request.evse_id.checked_sub(1).map(|index| index as usize);
        let index = index.filter(|&index| index < self.sessions.len() && request.connector_id == 1);
        let status = match index {
            None => UnlockStatus::UnknownConnector,
            Some(index) if self.authorized(index) && self.sessions[index].transaction.is_some() => {
                UnlockStatus::OngoingAuthorizedTransaction
            }
            Some(index) => {
                let evse = index as u32 + 1;
                self.emit(out, now, EventKind::ConnectorUnlocked { evse });
                UnlockStatus::Unlocked
            }
        };
        UnlockConnectorResponse {
            status,
            status_info: None,
        }
    }

    fn evse_for_remote_start(&self, evse: Option<u32>) -> Option<usize> {
        let free = |index: usize| {
            self.sessions[index].authorization == Authorization::None && self.operative(index)
        };
        match evse {
            Some(evse) => {
                let index = evse.checked_sub(1)? as usize;
                (index < self.sessions.len() && free(index)).then_some(index)
            }
            None => {
                let plugged = (0..self.sessions.len())
                    .find(|&index| free(index) && self.station.connectors[index].plugged.is_some());
                plugged.or_else(|| (0..self.sessions.len()).find(|&index| free(index)))
            }
        }
    }

    pub(super) fn authorized(&self, index: usize) -> bool {
        matches!(
            self.sessions[index].authorization,
            Authorization::Authorized { .. }
        )
    }
}

#[cfg(test)]
#[path = "remote_control.test.rs"]
mod tests;
