use super::ChargePoint;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::security::security_event::SecurityEventNotificationRequest;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::json;

pub const STARTUP_OF_THE_DEVICE: &str = "StartupOfTheDevice";
pub const RESET_OR_REBOOT: &str = "ResetOrReboot";
pub const FIRMWARE_UPDATED: &str = "FirmwareUpdated";
pub const INVALID_FIRMWARE_SIGNATURE: &str = "InvalidFirmwareSignature";
pub const INVALID_FIRMWARE_SIGNING_CERTIFICATE: &str = "InvalidFirmwareSigningCertificate";
pub const RECONFIGURATION_OF_SECURITY_PARAMETERS: &str = "ReconfigurationOfSecurityParameters";

impl ChargePoint {
    pub(super) fn security_event(&mut self, out: &mut Vec<Output>, now: u64, kind: &'static str) {
        let Ok(name) = CiString::new(kind) else {
            return;
        };
        let request = SecurityEventNotificationRequest {
            kind: name,
            timestamp: DateTime { unix_ms: now },
            tech_info: None,
        };
        self.outbox
            .push("SecurityEventNotification", 0, json!(request));
        let event_type = kind.into();
        self.emit(out, now, EventKind::SecurityEvent { event_type });
    }
}

#[cfg(test)]
#[path = "security_event.test.rs"]
mod tests;
