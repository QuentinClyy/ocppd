pub(super) mod charging_profiles;
pub(super) mod composite;

use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::provisioning::configuration::{
    CHARGE_PROFILE_MAX_STACK_LEVEL, CHARGING_SCHEDULE_MAX_PERIODS, MAX_CHARGING_PROFILES_INSTALLED,
};
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::charging_profile::{
    ChargingProfile, ChargingProfilePurpose,
};
use crate::protocol::v16::messages::smart_charging::{
    ChargingProfileStatus, ClearChargingProfileRequest, ClearChargingProfileStatus,
    SetChargingProfileRequest,
};

impl ChargePoint {
    pub(crate) fn set_charging_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: SetChargingProfileRequest,
    ) -> ChargingProfileStatus {
        let connector = request.connector_id;
        let profile = request.cs_charging_profiles;
        let known = connector == 0 || self.index_of(connector).is_some();
        let allowed = match profile.charging_profile_purpose {
            ChargingProfilePurpose::ChargePointMaxProfile => connector == 0,
            ChargingProfilePurpose::TxDefaultProfile => true,
            ChargingProfilePurpose::TxProfile => self
                .index_of(connector)
                .is_some_and(|index| self.runs_transaction(index, profile.transaction_id)),
        };
        if !(known && allowed && self.within_limits(&profile)) {
            return ChargingProfileStatus::Rejected;
        }
        let max = self.limit(MAX_CHARGING_PROFILES_INSTALLED) as usize;
        match self.install_profile(out, now, connector, profile, max) {
            true => ChargingProfileStatus::Accepted,
            false => ChargingProfileStatus::Rejected,
        }
    }

    pub(crate) fn fits_remote_start(&self, profile: &ChargingProfile) -> bool {
        profile.charging_profile_purpose == ChargingProfilePurpose::TxProfile
            && self.within_limits(profile)
    }

    pub(crate) fn attach_profile(&mut self, index: usize, profile: Option<ChargingProfile>) {
        if let Transaction::Starting { profile: slot, .. } = &mut self.transactions[index] {
            *slot = profile;
        }
    }

    pub(crate) fn install_transaction_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        transaction_id: i32,
        mut profile: ChargingProfile,
    ) {
        profile.transaction_id = Some(transaction_id);
        let accepted_with_the_remote_start = usize::MAX;
        self.install_profile(
            out,
            now,
            index as u32 + 1,
            profile,
            accepted_with_the_remote_start,
        );
    }

    fn install_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        connector: u32,
        profile: ChargingProfile,
        max: usize,
    ) -> bool {
        let kind = EventKind::ChargingProfileSet {
            connector,
            charging_profile_id: profile.charging_profile_id,
        };
        if !self.charging_profiles.install(connector, profile, max) {
            return false;
        }
        self.emit(out, now, kind);
        true
    }

    fn within_limits(&self, profile: &ChargingProfile) -> bool {
        profile.stack_level <= self.limit(CHARGE_PROFILE_MAX_STACK_LEVEL)
            && profile.charging_schedule.charging_schedule_period.len()
                <= self.limit(CHARGING_SCHEDULE_MAX_PERIODS) as usize
    }

    fn limit(&self, key: &str) -> u32 {
        self.configuration.number(key).unwrap_or(0)
    }

    pub(crate) fn clear_charging_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ClearChargingProfileRequest,
    ) -> ClearChargingProfileStatus {
        let count = self.charging_profiles.clear(&request);
        if count == 0 {
            return ClearChargingProfileStatus::Unknown;
        }
        self.emit(out, now, EventKind::ChargingProfilesCleared { count });
        ClearChargingProfileStatus::Accepted
    }

    fn runs_transaction(&self, index: usize, wanted: Option<i32>) -> bool {
        match self.transactions[index] {
            Transaction::Active { transaction_id, .. } => {
                wanted.is_none() || wanted == transaction_id
            }
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
