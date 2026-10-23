use crate::protocol::shared::smart_charging::schedule::{DAY_MS, Kind, Period, Schedule, Unit};
use crate::protocol::v16::messages::datatypes::charging_profile::{
    ChargingProfile, ChargingProfileKind, ChargingProfilePurpose, ChargingRateUnit, RecurrencyKind,
};
use crate::protocol::v16::messages::smart_charging::ClearChargingProfileRequest;

pub struct Installed {
    pub connector: u32,
    pub profile: ChargingProfile,
    pub schedule: Schedule,
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
        let schedule = schedule(&profile);
        self.0.push(Installed {
            connector,
            profile,
            schedule,
        });
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

    pub fn stacked(&self, connector: u32, purpose: ChargingProfilePurpose) -> Vec<&Schedule> {
        let mut installed: Vec<_> = self
            .0
            .iter()
            .filter(|installed| installed.connector == connector)
            .filter(|installed| installed.profile.charging_profile_purpose == purpose)
            .collect();
        installed.sort_by_key(|installed| std::cmp::Reverse(installed.profile.stack_level));
        installed
            .into_iter()
            .map(|installed| &installed.schedule)
            .collect()
    }

    pub fn for_connector(&self, connector: u32) -> impl Iterator<Item = &Schedule> {
        self.0
            .iter()
            .filter(move |installed| installed.connector == 0 || installed.connector == connector)
            .map(|installed| &installed.schedule)
    }

    pub fn end_transaction(&mut self, connector: u32) {
        self.0.retain(|installed| {
            installed.connector != connector
                || installed.profile.charging_profile_purpose != ChargingProfilePurpose::TxProfile
        });
    }
}

fn schedule(profile: &ChargingProfile) -> Schedule {
    let schedule = &profile.charging_schedule;
    let kind = match profile.charging_profile_kind {
        ChargingProfileKind::Absolute => Kind::Absolute,
        ChargingProfileKind::Relative => Kind::Relative,
        ChargingProfileKind::Recurring => Kind::Recurring {
            cycle_ms: profile.recurrency_kind.map(|recurrency| match recurrency {
                RecurrencyKind::Daily => DAY_MS,
                RecurrencyKind::Weekly => 7 * DAY_MS,
            }),
        },
    };
    let unit = match schedule.charging_rate_unit {
        ChargingRateUnit::W => Unit::Watts,
        ChargingRateUnit::A => Unit::Amps,
    };
    let periods = schedule.charging_schedule_period.iter();
    Schedule {
        valid_from: profile.valid_from.map(|from| from.unix_ms),
        valid_to: profile.valid_to.map(|to| to.unix_ms),
        kind,
        start: schedule.start_schedule.map(|start| start.unix_ms),
        duration_ms: schedule.duration.map(ms),
        unit,
        periods: periods
            .map(|period| Period {
                start_ms: ms(period.start_period),
                limit: period.limit,
                phases: period.number_phases,
            })
            .collect(),
    }
}

fn ms(seconds: u32) -> u64 {
    u64::from(seconds) * 1000
}

#[cfg(test)]
#[path = "charging_profiles.test.rs"]
mod tests;
