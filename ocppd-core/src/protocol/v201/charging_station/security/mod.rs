pub(super) mod certificates;
pub(super) mod client_certificate;

use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::security::SecurityEventNotificationRequest;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::json;

pub const SECURITY_CTRLR: &str = "SecurityCtrlr";
pub const SECURITY_PROFILE: &str = "SecurityProfile";
pub const BASIC_AUTH_PASSWORD: &str = "BasicAuthPassword";
pub const CERTIFICATE_ENTRIES: &str = "CertificateEntries";
pub const ORGANIZATION_NAME: &str = "OrganizationName";
pub const CERT_SIGNING_WAIT_MINIMUM: &str = "CertSigningWaitMinimum";
pub const CERT_SIGNING_REPEAT_TIMES: &str = "CertSigningRepeatTimes";
pub const STARTUP_OF_THE_DEVICE: &str = "StartupOfTheDevice";
pub const RESET_OR_REBOOT: &str = "ResetOrReboot";
pub const RECONFIGURATION_OF_SECURITY_PARAMETERS: &str = "ReconfigurationOfSecurityParameters";
pub const FIRMWARE_UPDATED: &str = "FirmwareUpdated";
pub const INVALID_FIRMWARE_SIGNATURE: &str = "InvalidFirmwareSignature";
pub const INVALID_FIRMWARE_SIGNING_CERTIFICATE: &str = "InvalidFirmwareSigningCertificate";
pub const INVALID_CHARGING_STATION_CERTIFICATE: &str = "InvalidChargingStationCertificate";
const CRITICAL: [&str; 6] = [
    STARTUP_OF_THE_DEVICE,
    RESET_OR_REBOOT,
    FIRMWARE_UPDATED,
    INVALID_FIRMWARE_SIGNATURE,
    INVALID_FIRMWARE_SIGNING_CERTIFICATE,
    INVALID_CHARGING_STATION_CERTIFICATE,
];
const BASIC_AUTH_PROFILES: [u64; 2] = [1, 2];
const TLS_FROM_PROFILE: u64 = 2;
const CLIENT_CERTIFICATE_PROFILE: u64 = 3;

impl ChargingStation {
    pub(super) fn security_event(&mut self, out: &mut Vec<Output>, now: u64, kind: &'static str) {
        let event_type = kind.to_owned();
        self.emit(out, now, EventKind::SecurityEvent { event_type });
        let Ok(name) = CiString::new(kind) else {
            return;
        };
        if CRITICAL.contains(&kind) {
            let request = SecurityEventNotificationRequest {
                kind: name,
                timestamp: DateTime { unix_ms: now },
                tech_info: None,
            };
            self.outbox
                .push("SecurityEventNotification", 0, json!(request));
        }
    }

    pub fn basic_auth_password(&self) -> Option<Vec<u8>> {
        let password = self
            .device_model
            .value(SECURITY_CTRLR, BASIC_AUTH_PASSWORD)?;
        let profile = self.security_profile();
        Some(password.as_bytes().to_vec())
            .filter(|password| !password.is_empty() && BASIC_AUTH_PROFILES.contains(&profile))
    }

    pub fn requires_tls(&self) -> bool {
        self.security_profile() >= TLS_FROM_PROFILE
    }

    pub fn requires_client_certificate(&self) -> bool {
        self.security_profile() == CLIENT_CERTIFICATE_PROFILE
    }

    fn security_profile(&self) -> u64 {
        self.device_model
            .number(SECURITY_CTRLR, SECURITY_PROFILE)
            .unwrap_or(0)
    }
}

pub fn survives_disconnect(action: &str) -> bool {
    action == "SecurityEventNotification"
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
