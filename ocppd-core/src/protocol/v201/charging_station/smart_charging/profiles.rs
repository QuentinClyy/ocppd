use crate::protocol::v201::messages::datatypes::charging_profile::{
    ChargingProfile, ChargingProfileKind, ChargingProfilePurpose, ChargingRateUnit, RecurrencyKind,
};
use crate::protocol::v201::messages::smart_charging::clear::ClearChargingProfileRequest;
use crate::protocol::v201::messages::smart_charging::report::{
    ChargingLimitSource, ChargingProfileCriterion,
};
use crate::smart_charging::schedule::{DAY_MS, Kind, Period, Schedule, Unit};

struct Installed {
    evse: u32,
    profile: ChargingProfile,
    schedule: Schedule,
}

#[derive(Default)]
pub struct ChargingProfiles(Vec<Installed>);

impl ChargingProfiles {
    pub fn install(
        &mut self,
        evse: u32,
        profile: ChargingProfile,
        schedule: Schedule,
        max: usize,
    ) -> bool {
        let replaced = |installed: &Installed| {
            let old = &installed.profile;
            old.id == profile.id
                || (installed.evse == evse
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
        self.0.push(Installed {
            evse,
            profile,
            schedule,
        });
        true
    }

    pub fn duplicate_default(&self, evse: u32, profile: &ChargingProfile) -> bool {
        self.0.iter().any(|installed| {
            let other = &installed.profile;
            (installed.evse == 0) != (evse == 0)
                && other.charging_profile_purpose == ChargingProfilePurpose::TxDefaultProfile
                && other.stack_level == profile.stack_level
                && other.id != profile.id
        })
    }

    pub fn stacked(&self, evse: u32, purpose: ChargingProfilePurpose) -> Vec<&Schedule> {
        let mut installed: Vec<_> = self
            .0
            .iter()
            .filter(|installed| installed.evse == evse)
            .filter(|installed| installed.profile.charging_profile_purpose == purpose)
            .collect();
        installed.sort_by_key(|installed| std::cmp::Reverse(installed.profile.stack_level));
        installed
            .into_iter()
            .map(|installed| &installed.schedule)
            .collect()
    }

    pub fn for_evse(&self, evse: u32) -> impl Iterator<Item = &Schedule> {
        self.0
            .iter()
            .filter(move |installed| installed.evse == 0 || installed.evse == evse)
            .map(|installed| &installed.schedule)
    }

    pub fn end_transaction(&mut self, evse: u32) -> bool {
        let before = self.0.len();
        self.0.retain(|installed| {
            installed.evse != evse
                || installed.profile.charging_profile_purpose != ChargingProfilePurpose::TxProfile
        });
        before != self.0.len()
    }

    pub fn matching(
        &self,
        evse: Option<u32>,
        criterion: &ChargingProfileCriterion,
    ) -> Vec<(u32, &ChargingProfile)> {
        let ours = criterion
            .charging_limit_source
            .as_ref()
            .is_none_or(|sources| sources.contains(&ChargingLimitSource::Cso));
        let matches = |installed: &&Installed| {
            let profile = &installed.profile;
            let fields = match &criterion.charging_profile_id {
                Some(ids) => ids.contains(&profile.id),
                None => {
                    criterion
                        .charging_profile_purpose
                        .is_none_or(|purpose| purpose == profile.charging_profile_purpose)
                        && criterion
                            .stack_level
                            .is_none_or(|level| level == profile.stack_level)
                }
            };
            ours && evse.is_none_or(|evse| evse == installed.evse) && fields
        };
        self.0
            .iter()
            .filter(matches)
            .map(|installed| (installed.evse, &installed.profile))
            .collect()
    }

    pub fn clear(&mut self, request: &ClearChargingProfileRequest) -> usize {
        let matches = |installed: &Installed| {
            let profile = &installed.profile;
            if let Some(id) = request.charging_profile_id {
                return profile.id == id;
            }
            let Some(criteria) = &request.charging_profile_criteria else {
                return true;
            };
            criteria.evse_id.is_none_or(|evse| evse == installed.evse)
                && criteria
                    .charging_profile_purpose
                    .is_none_or(|purpose| purpose == profile.charging_profile_purpose)
                && criteria
                    .stack_level
                    .is_none_or(|level| level == profile.stack_level)
        };
        let before = self.0.len();
        self.0.retain(|installed| !matches(installed));
        before - self.0.len()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn schedule(profile: &ChargingProfile) -> Option<Schedule> {
    let [schedule] = profile.charging_schedule.as_slice() else {
        return None;
    };
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
    Some(Schedule {
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
    })
}

fn ms(seconds: u32) -> u64 {
    u64::from(seconds) * 1000
}

#[cfg(test)]
#[path = "profiles.test.rs"]
mod tests;
