pub(super) mod fault;

use super::ChargePoint;
use crate::api::io::Output;
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::{
    ChargePointErrorCode, ChargePointStatus, StatusNotificationRequest,
};
use crate::protocol::v16::messages::remote_control::{
    AvailabilityStatus, AvailabilityType, ChangeAvailabilityRequest,
};
use crate::station::connector::Fault;
use serde_json::json;

impl ChargePoint {
    fn error_code(&self, connector_id: u32) -> ChargePointErrorCode {
        let index = connector_id.checked_sub(1).map(|index| index as usize);
        let fault = index.and_then(|index| self.station.connectors.get(index)?.fault);
        match fault {
            Some(Fault::Internal) => ChargePointErrorCode::InternalError,
            Some(Fault::ConnectorLock) => ChargePointErrorCode::ConnectorLockFailure,
            None => ChargePointErrorCode::NoError,
        }
    }

    pub(super) fn status_notification(&mut self, connector_id: u32, status: ChargePointStatus) {
        let request = StatusNotificationRequest {
            connector_id,
            error_code: self.error_code(connector_id),
            info: None,
            status,
            timestamp: None,
            vendor_id: None,
            vendor_error_code: None,
        };
        self.outbox
            .push("StatusNotification", connector_id, json!(request));
    }

    pub(crate) fn change_availability(
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
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
