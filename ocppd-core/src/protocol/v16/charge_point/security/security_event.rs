use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::security::security_event::SecurityEventNotificationRequest;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::{Value, json};

pub const STARTUP_OF_THE_DEVICE: &str = "StartupOfTheDevice";
pub const RESET_OR_REBOOT: &str = "ResetOrReboot";
pub const FIRMWARE_UPDATED: &str = "FirmwareUpdated";
pub const INVALID_FIRMWARE_SIGNATURE: &str = "InvalidFirmwareSignature";
pub const INVALID_FIRMWARE_SIGNING_CERTIFICATE: &str = "InvalidFirmwareSigningCertificate";
pub const INVALID_CHARGE_POINT_CERTIFICATE: &str = "InvalidChargePointCertificate";
pub const RECONFIGURATION_OF_SECURITY_PARAMETERS: &str = "ReconfigurationOfSecurityParameters";
const CRITICAL: [&str; 5] = [
    STARTUP_OF_THE_DEVICE,
    RESET_OR_REBOOT,
    FIRMWARE_UPDATED,
    INVALID_FIRMWARE_SIGNING_CERTIFICATE,
    INVALID_CHARGE_POINT_CERTIFICATE,
];

impl ChargePoint {
    pub(crate) fn security_event(&mut self, out: &mut Vec<Output>, now: u64, kind: &'static str) {
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

pub fn is_critical(notification: &Value) -> bool {
    let kind = notification["type"].as_str().unwrap_or_default();
    CRITICAL.contains(&kind)
}

#[cfg(test)]
#[path = "security_event.test.rs"]
mod tests;
