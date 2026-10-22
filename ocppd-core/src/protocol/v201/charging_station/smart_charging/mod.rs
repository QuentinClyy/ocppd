mod clear;
mod composite;
mod limit;
pub(super) mod profiles;
mod report;

use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::charging_profile::{
    ChargingProfile, ChargingProfileKind, ChargingProfilePurpose,
};
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::protocol::v201::messages::smart_charging::{
    ChargingProfileStatus, SetChargingProfileRequest, SetChargingProfileResponse,
};
use crate::wire::ci_string::CiString;

pub const SMART_CHARGING_CTRLR: &str = "SmartChargingCtrlr";
pub const ENTRIES: &str = "Entries";
pub const ENTRIES_INSTANCE: &str = "ChargingProfiles";
pub const PROFILE_STACK_LEVEL: &str = "ProfileStackLevel";
pub const PERIODS_PER_SCHEDULE: &str = "PeriodsPerSchedule";
pub const MAX_PROFILES: usize = 20;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn set_charging_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: SetChargingProfileRequest,
    ) -> SetChargingProfileResponse {
        let evse = request.evse_id;
        let profile = request.charging_profile;
        if let Some(reason) = self.refusal(evse, &profile) {
            return response(ChargingProfileStatus::Rejected, Some(reason));
        }
        let Some(schedule) = profiles::schedule(&profile) else {
            return response(ChargingProfileStatus::Rejected, Some("InvalidSchedule"));
        };
        let kind = EventKind::ChargingProfileSet {
            evse,
            charging_profile_id: profile.id,
        };
        if !self
            .charging_profiles
            .install(evse, profile, schedule, MAX_PROFILES)
        {
            return response(ChargingProfileStatus::Rejected, Some("TooManyProfiles"));
        }
        self.count_profiles();
        self.emit(out, now, kind);
        response(ChargingProfileStatus::Accepted, None)
    }

    pub(in crate::protocol::v201::charging_station) fn remote_start_refusal(
        &self,
        profile: &ChargingProfile,
    ) -> Option<&'static str> {
        let purpose = profile.charging_profile_purpose;
        if purpose != ChargingProfilePurpose::TxProfile || profile.transaction_id.is_some() {
            return Some("InvalidProfile");
        }
        self.invalid_schedule(profile).then_some("InvalidSchedule")
    }

    pub(in crate::protocol::v201::charging_station) fn attach_remote_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
    ) {
        let session = &mut self.sessions[index];
        let Some(transaction) = &session.transaction else {
            return;
        };
        let Some(mut profile) = session.remote_profile.take() else {
            return;
        };
        profile.transaction_id = Some(transaction.id.clone());
        let Some(schedule) = profiles::schedule(&profile) else {
            return;
        };
        let evse = index as u32 + 1;
        let kind = EventKind::ChargingProfileSet {
            evse,
            charging_profile_id: profile.id,
        };
        let accepted_with_the_remote_start = usize::MAX;
        self.charging_profiles
            .install(evse, profile, schedule, accepted_with_the_remote_start);
        self.count_profiles();
        self.emit(out, now, kind);
    }

    pub(in crate::protocol::v201::charging_station) fn count_profiles(&mut self) {
        let count = self.charging_profiles.len().to_string();
        self.device_model.set_instance_value(
            SMART_CHARGING_CTRLR,
            ENTRIES,
            ENTRIES_INSTANCE,
            count,
        );
    }

    fn refusal(&self, evse: u32, profile: &ChargingProfile) -> Option<&'static str> {
        if evse as usize > self.sessions.len() {
            return Some("UnknownEVSE");
        }
        let transaction = evse
            .checked_sub(1)
            .and_then(|index| self.sessions[index as usize].transaction.as_ref());
        let refused = match profile.charging_profile_purpose {
            ChargingProfilePurpose::ChargingStationExternalConstraints => Some("InvalidProfile"),
            ChargingProfilePurpose::ChargingStationMaxProfile => (evse != 0
                || profile.charging_profile_kind == ChargingProfileKind::Relative)
                .then_some("InvalidProfile"),
            ChargingProfilePurpose::TxProfile if evse == 0 => Some("InvalidProfile"),
            ChargingProfilePurpose::TxProfile => {
                let wanted = profile.transaction_id.as_ref().map(CiString::as_str);
                let running = transaction.map(|transaction| transaction.id.as_str());
                (running.is_none() || wanted != running).then_some("TxNotFound")
            }
            ChargingProfilePurpose::TxDefaultProfile => self
                .charging_profiles
                .duplicate_default(evse, profile)
                .then_some("DuplicateProfile"),
        };
        refused.or_else(|| self.invalid_schedule(profile).then_some("InvalidSchedule"))
    }

    fn invalid_schedule(&self, profile: &ChargingProfile) -> bool {
        let setting = |name| {
            let number = self.device_model.number(SMART_CHARGING_CTRLR, name);
            number.unwrap_or(0)
        };
        let [schedule] = profile.charging_schedule.as_slice() else {
            return true;
        };
        let periods = &schedule.charging_schedule_period;
        let starts_at_zero = periods.first().is_some_and(|first| first.start_period == 0);
        let ordered = periods
            .windows(2)
            .all(|pair| pair[0].start_period < pair[1].start_period);
        let start_wanted = profile.charging_profile_kind != ChargingProfileKind::Relative;
        let recurring = profile.charging_profile_kind == ChargingProfileKind::Recurring;
        !starts_at_zero
            || !ordered
            || periods.iter().any(|period| period.phase_to_use.is_some())
            || schedule.start_schedule.is_some() != start_wanted
            || profile.recurrency_kind.is_some() != recurring
            || u64::from(profile.stack_level) > setting(PROFILE_STACK_LEVEL)
            || periods.len() as u64 > setting(PERIODS_PER_SCHEDULE)
    }
}

fn response(status: ChargingProfileStatus, reason: Option<&str>) -> SetChargingProfileResponse {
    let status_info = reason.and_then(|reason| {
        Some(StatusInfo {
            reason_code: CiString::new(reason).ok()?,
            additional_info: None,
        })
    });
    SetChargingProfileResponse {
        status,
        status_info,
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
