use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::provisioning::configuration::{
    AUTHORIZATION_KEY, SECURITY_PROFILE, hex, hex_encode,
};
use crate::protocol::v16::messages::provisioning::configuration::{
    ChangeConfigurationRequest, ConfigurationStatus,
};
use crate::wire::ci_string::CiString;

const BASIC_AUTH_PROFILES: [u32; 2] = [1, 2];
const TLS_FROM_PROFILE: u32 = 2;
const CLIENT_CERTIFICATE_PROFILE: u32 = 3;

impl ChargePoint {
    pub fn configure(&mut self, key: &str, value: &str) -> ConfigurationStatus {
        let (Ok(key), Ok(value)) = (CiString::new(key), CiString::new(value)) else {
            return ConfigurationStatus::Rejected;
        };
        self.configuration
            .install(&ChangeConfigurationRequest { key, value })
    }

    pub fn set_basic_auth_password(&mut self, password: &str) -> bool {
        if password.is_empty() {
            return false;
        }
        let Ok(key) = CiString::new(&hex_encode(password.as_bytes())) else {
            return false;
        };
        self.configuration.set_text(AUTHORIZATION_KEY, key);
        true
    }

    pub fn basic_auth_password(&self) -> Option<Vec<u8>> {
        let profile = self.configuration.number(SECURITY_PROFILE)?;
        let key = hex(self.configuration.text(AUTHORIZATION_KEY)?)?;
        Some(key).filter(|key| BASIC_AUTH_PROFILES.contains(&profile) && !key.is_empty())
    }

    pub fn requires_tls(&self) -> bool {
        let profile = self.configuration.number(SECURITY_PROFILE);
        profile.is_some_and(|profile| profile >= TLS_FROM_PROFILE)
    }

    pub fn requires_client_certificate(&self) -> bool {
        let profile = self.configuration.number(SECURITY_PROFILE);
        profile == Some(CLIENT_CERTIFICATE_PROFILE)
    }

    pub(crate) fn security_prerequisites_met(&self, request: &ChangeConfigurationRequest) -> bool {
        if !request.key.as_str().eq_ignore_ascii_case(SECURITY_PROFILE) {
            return true;
        }
        let Ok(profile) = request.value.as_str().parse::<u32>() else {
            return true;
        };
        let has_key = self
            .configuration
            .text(AUTHORIZATION_KEY)
            .is_some_and(|key| !key.is_empty());
        let has_root = !self.csms_roots().is_empty();
        let has_certificate = self.has_charge_point_certificate();
        (!BASIC_AUTH_PROFILES.contains(&profile) || has_key)
            && (profile < TLS_FROM_PROFILE || has_root)
            && (profile != CLIENT_CERTIFICATE_PROFILE || has_certificate)
    }

    pub(crate) fn reconnect_due(&mut self, now: u64) -> bool {
        let due = self.reconnect_requested && self.outbox.is_drained(now);
        if due {
            self.reconnect_requested = false;
        }
        due
    }
}

#[cfg(test)]
#[path = "security_profile.test.rs"]
mod tests;
