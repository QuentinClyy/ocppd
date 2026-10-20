use super::ChargePoint;
use super::registration::Registration;
use super::transaction::Transaction;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::core::remote_control::{
    AvailabilityStatus, AvailabilityType, ChangeAvailabilityRequest, RemoteStartTransactionRequest,
    RemoteStopTransactionRequest, ResetRequest, ResetType, UnlockConnectorRequest, UnlockStatus,
};
use crate::v16::messages::core::transaction::Reason;
use crate::v16::messages::datatypes::generic_status::GenericStatus;

impl ChargePoint {
    pub(super) fn remote_start(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: RemoteStartTransactionRequest,
    ) -> GenericStatus {
        let usable = |index: usize| {
            self.transactions[index] == Transaction::Idle && !self.unavailable(index)
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
        let id_tag = request.id_tag.as_str().to_owned();
        match connector {
            Some(index) if plugged(index) => self.start_transaction(now, index, request.id_tag),
            _ => self.hold_authorized(now, request.id_tag, connector),
        }
        let kind = EventKind::RemoteStart {
            connector: connector.map(|index| index as u32 + 1),
            id_tag,
            expires: self.held_deadline(),
        };
        self.emit(out, now, kind);
        GenericStatus::Accepted
    }

    pub(super) fn remote_stop(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: RemoteStopTransactionRequest,
    ) -> GenericStatus {
        let running = self.transactions.iter().position(|transaction| {
            matches!(transaction, Transaction::Active { transaction_id, .. }
                if *transaction_id == request.transaction_id)
        });
        let Some(index) = running else {
            return GenericStatus::Rejected;
        };
        self.stop(out, now, index, Reason::Remote);
        GenericStatus::Accepted
    }

    pub(super) fn unlock(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: UnlockConnectorRequest,
    ) -> UnlockStatus {
        let Some(index) = self.index_of(request.connector_id) else {
            return UnlockStatus::NotSupported;
        };
        self.stop(out, now, index, Reason::UnlockCommand);
        UnlockStatus::Unlocked
    }

    pub(super) fn reset(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ResetRequest,
    ) -> GenericStatus {
        let reason = match request.kind {
            ResetType::Soft => Reason::SoftReset,
            ResetType::Hard => Reason::HardReset,
        };
        for index in 0..self.transactions.len() {
            match self.transactions[index] {
                Transaction::Authorizing { .. } | Transaction::Starting { .. } => {
                    self.transactions[index] = Transaction::Idle
                }
                _ => self.stop(out, now, index, reason),
            }
        }
        self.emit(
            out,
            now,
            EventKind::Reset {
                reset_type: request.kind,
            },
        );
        self.held = None;
        self.resetting = Some(request.kind);
        GenericStatus::Accepted
    }

    pub(super) fn reset_due(&self, now: u64) -> bool {
        match self.resetting {
            Some(ResetType::Hard) => true,
            Some(ResetType::Soft) => self.outbox.is_drained(now),
            None => false,
        }
    }

    pub(super) fn rebooted(&mut self) {
        self.resetting = None;
        self.registration = Registration::Offline;
        self.outbox.keep_transaction_messages();
        self.connection_lost();
        for transaction in &mut self.transactions {
            if *transaction == Transaction::Finished {
                *transaction = Transaction::Idle;
            }
        }
    }

    pub(super) fn change_availability(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ChangeAvailabilityRequest,
    ) -> AvailabilityStatus {
        let connector = request.connector_id as usize;
        if connector >= self.inoperative.len() {
            return AvailabilityStatus::Rejected;
        }
        let operative = request.kind == AvailabilityType::Operative;
        self.inoperative[connector] = !operative;
        let kind = EventKind::AvailabilityChanged {
            connector: request.connector_id,
            operative,
        };
        self.emit(out, now, kind);
        let affected = match connector {
            0 => 0..self.transactions.len(),
            _ => connector - 1..connector,
        };
        let busy = affected.into_iter().any(|index| {
            !matches!(
                self.transactions[index],
                Transaction::Idle | Transaction::Finished
            )
        });
        if busy && !operative {
            AvailabilityStatus::Scheduled
        } else {
            AvailabilityStatus::Accepted
        }
    }

    pub(super) fn index_of(&self, connector: u32) -> Option<usize> {
        connector
            .checked_sub(1)
            .map(|index| index as usize)
            .filter(|&index| index < self.transactions.len())
    }
}

#[cfg(test)]
#[path = "remote_control.test.rs"]
mod tests;
