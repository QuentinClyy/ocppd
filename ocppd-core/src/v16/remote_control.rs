use super::charge_point::ChargePoint;
use super::messages::generic_status::GenericStatus;
use super::messages::remote_control::{
    AvailabilityStatus, AvailabilityType, ChangeAvailabilityRequest, RemoteStartTransactionRequest,
    RemoteStopTransactionRequest, ResetRequest, ResetType, UnlockConnectorRequest, UnlockStatus,
};
use super::messages::transaction::Reason;
use super::transaction::Transaction;
use crate::event::{EventKind, Output};

impl ChargePoint {
    pub(super) fn remote_start(
        &mut self,
        now: u64,
        request: RemoteStartTransactionRequest,
    ) -> GenericStatus {
        let ready = |index: usize| {
            self.transactions[index] == Transaction::Idle
                && self.station.connectors[index].plugged.is_some()
                && !self.unavailable(index)
        };
        let index = match request.connector_id {
            Some(connector) => self.index_of(connector).filter(|&index| ready(index)),
            None => (0..self.transactions.len()).find(|&index| ready(index)),
        };
        let Some(index) = index else {
            return GenericStatus::Rejected;
        };
        self.start_transaction(now, index, request.id_tag);
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
            self.stop(out, now, index, reason);
        }
        self.emit(
            out,
            now,
            EventKind::Reset {
                reset_type: request.kind,
            },
        );
        self.resetting = true;
        GenericStatus::Accepted
    }

    pub(super) fn rebooted(&mut self) {
        self.resetting = false;
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
