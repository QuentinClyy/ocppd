pub(super) mod trigger;

use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::provisioning::configuration::AUTHORIZE_REMOTE_TX_REQUESTS;
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v16::messages::remote_control::{
    RemoteStartTransactionRequest, RemoteStopTransactionRequest, UnlockConnectorRequest,
    UnlockStatus,
};
use crate::protocol::v16::messages::transaction::Reason;

impl ChargePoint {
    pub(crate) fn remote_start(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: RemoteStartTransactionRequest,
    ) -> GenericStatus {
        let usable = |index: usize| {
            self.transactions[index] == Transaction::Idle
                && !self.unavailable(index)
                && !self.reserved_for_other(index, &request.id_tag, None)
        };
        let plugged = |index: usize| self.station.connectors[index].plugged.is_some();
        let target = match request.connector_id {
            Some(connector) => self
                .index_of(connector)
                .filter(|&index| usable(index))
                .map(Some),
            None => {
                let ready =
                    (0..self.transactions.len()).find(|&index| usable(index) && plugged(index));
                let any = (0..self.transactions.len()).any(usable);
                ready.map(Some).or(any.then_some(None))
            }
        };
        let (Some(connector), None, None) = (target, &self.held, self.resetting) else {
            return GenericStatus::Rejected;
        };
        if !request
            .charging_profile
            .as_ref()
            .is_none_or(|profile| self.fits_remote_start(profile))
        {
            return GenericStatus::Rejected;
        }
        let id_tag = request.id_tag.as_str().to_owned();
        let profile = request.charging_profile;
        match (
            connector,
            self.configuration.flag(AUTHORIZE_REMOTE_TX_REQUESTS),
        ) {
            (Some(index), true) if plugged(index) => {
                self.authorize_on(out, now, index, request.id_tag, profile)
            }
            (_, true) => self.hold_for_authorization(out, now, request.id_tag, connector, profile),
            (Some(index), false) if plugged(index) => {
                self.start_transaction(out, now, index, request.id_tag);
                self.attach_profile(index, profile);
            }
            (_, false) => self.hold_authorized(now, request.id_tag, connector, profile),
        }
        let kind = EventKind::RemoteStart {
            connector: connector.map(|index| index as u32 + 1),
            id_tag,
            expires: self.held_deadline(),
        };
        self.emit(out, now, kind);
        GenericStatus::Accepted
    }

    pub(crate) fn remote_stop(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: RemoteStopTransactionRequest,
    ) -> GenericStatus {
        let running = self.transactions.iter().position(|transaction| {
            matches!(transaction, Transaction::Active { transaction_id, .. }
                if *transaction_id == Some(request.transaction_id))
        });
        let Some(index) = running else {
            return GenericStatus::Rejected;
        };
        self.stop(out, now, index, Reason::Remote);
        GenericStatus::Accepted
    }

    pub(crate) fn unlock(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: UnlockConnectorRequest,
    ) -> UnlockStatus {
        let Some(index) = self.index_of(request.connector_id) else {
            return UnlockStatus::NotSupported;
        };
        if self.station.connectors[index].lock_broken {
            return UnlockStatus::UnlockFailed;
        }
        self.stop(out, now, index, Reason::UnlockCommand);
        UnlockStatus::Unlocked
    }

    pub(crate) fn index_of(&self, connector: u32) -> Option<usize> {
        connector
            .checked_sub(1)
            .map(|index| index as usize)
            .filter(|&index| index < self.transactions.len())
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
