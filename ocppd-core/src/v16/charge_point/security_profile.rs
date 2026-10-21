use super::ChargePoint;
use super::configuration::{AUTHORIZATION_KEY, SECURITY_PROFILE, hex};
use crate::v16::messages::core::configuration::{ChangeConfigurationRequest, ConfigurationStatus};
use crate::wire::ci_string::CiString;

const BASIC_AUTH_PROFILES: [u32; 2] = [1, 2];
const TLS_FROM_PROFILE: u32 = 2;

impl ChargePoint {
    pub fn configure(&mut self, key: &str, value: &str) -> ConfigurationStatus {
        let (Ok(key), Ok(value)) = (CiString::new(key), CiString::new(value)) else {
            return ConfigurationStatus::Rejected;
        };
        self.configuration
            .change(&ChangeConfigurationRequest { key, value })
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
}

#[cfg(test)]
#[path = "security_profile.test.rs"]
mod tests;
