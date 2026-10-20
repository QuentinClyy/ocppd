use crate::v16::messages::datatypes::charging_profile::{ChargingProfile, ChargingProfilePurpose};
use crate::v16::messages::smart_charging::ClearChargingProfileRequest;

pub struct Installed {
    pub connector: u32,
    pub profile: ChargingProfile,
}

#[derive(Default)]
pub struct ChargingProfiles(Vec<Installed>);

impl ChargingProfiles {
    pub fn install(&mut self, connector: u32, profile: ChargingProfile, max: usize) -> bool {
        let replaced = |installed: &Installed| {
            let old = &installed.profile;
            old.charging_profile_id == profile.charging_profile_id
                || (installed.connector == connector
                    && old.charging_profile_purpose == profile.charging_profile_purpose
                    && old.stack_level == profile.stack_level)
        };
        let kept = self
            .0
            .iter()
            .filter(|installed| !replaced(installed))
            .count();
        if kept >= max {
            return false;
        }
        self.0.retain(|installed| !replaced(installed));
        self.0.push(Installed { connector, profile });
        true
    }

    pub fn clear(&mut self, request: &ClearChargingProfileRequest) -> usize {
        let matches = |installed: &Installed| {
            let profile = &installed.profile;
            if let Some(id) = request.id {
                return profile.charging_profile_id == id;
            }
            request
                .connector_id
                .is_none_or(|connector| connector == installed.connector)
                && request
                    .charging_profile_purpose
                    .is_none_or(|purpose| purpose == profile.charging_profile_purpose)
                && request
                    .stack_level
                    .is_none_or(|level| level == profile.stack_level)
        };
        let before = self.0.len();
        self.0.retain(|installed| !matches(installed));
        before - self.0.len()
    }

    pub fn end_transaction(&mut self, connector: u32) {
        self.0.retain(|installed| {
            installed.connector != connector
                || installed.profile.charging_profile_purpose != ChargingProfilePurpose::TxProfile
        });
    }
}

#[cfg(test)]
#[path = "charging_profiles.test.rs"]
mod tests;
