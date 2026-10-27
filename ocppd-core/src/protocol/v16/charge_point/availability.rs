use super::ChargePoint;
use crate::protocol::v16::messages::availability::{
    ChargePointErrorCode, ChargePointStatus, StatusNotificationRequest,
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
}
