use super::ChargePoint;
use super::configuration::{
    CHARGE_PROFILE_MAX_STACK_LEVEL, CHARGING_SCHEDULE_MAX_PERIODS, MAX_CHARGING_PROFILES_INSTALLED,
};
use super::transaction::Transaction;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::datatypes::charging_profile::ChargingProfilePurpose;
use crate::v16::messages::smart_charging::{
    ChargingProfileStatus, ClearChargingProfileRequest, ClearChargingProfileStatus,
    SetChargingProfileRequest,
};

impl ChargePoint {
    pub(super) fn set_charging_profile(
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
        let limit = |key| self.configuration.number(key).unwrap_or(0);
        let within_limits = profile.stack_level <= limit(CHARGE_PROFILE_MAX_STACK_LEVEL)
            && profile.charging_schedule.charging_schedule_period.len()
                <= limit(CHARGING_SCHEDULE_MAX_PERIODS) as usize;
        if !(known && allowed && within_limits) {
            return ChargingProfileStatus::Rejected;
        }
        let max = limit(MAX_CHARGING_PROFILES_INSTALLED) as usize;
        let kind = EventKind::ChargingProfileSet {
            connector,
            charging_profile_id: profile.charging_profile_id,
        };
        if !self.charging_profiles.install(connector, profile, max) {
            return ChargingProfileStatus::Rejected;
        }
        self.emit(out, now, kind);
        ChargingProfileStatus::Accepted
    }

    pub(super) fn clear_charging_profile(
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
