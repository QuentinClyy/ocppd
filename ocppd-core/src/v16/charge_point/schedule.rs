use crate::station::Station;
use crate::v16::messages::datatypes::charging_profile::{
    ChargingProfile, ChargingProfileKind, ChargingRateUnit, RecurrencyKind,
};

const DAY_MS: u64 = 86_400_000;
const DEFAULT_PHASES: u32 = 3;

pub fn limit_w(
    profile: &ChargingProfile,
    at: u64,
    transaction_start: Option<u64>,
    station: &Station,
) -> Option<f64> {
    if !valid_at(profile, at) {
        return None;
    }
    let schedule = &profile.charging_schedule;
    let start = cycle_start(profile, at, transaction_start).filter(|&start| start <= at)?;
    let elapsed_ms = at - start;
    if schedule
        .duration
        .is_some_and(|seconds| elapsed_ms >= ms(seconds))
    {
        return None;
    }
    let period = schedule
        .charging_schedule_period
        .iter()
        .filter(|period| ms(period.start_period) <= elapsed_ms)
        .max_by_key(|period| period.start_period)?;
    Some(match schedule.charging_rate_unit {
        ChargingRateUnit::W => period.limit,
        ChargingRateUnit::A => {
            let phases = period
                .number_phases
                .unwrap_or(DEFAULT_PHASES)
                .min(station.phases);
            period.limit * station.voltage_v * f64::from(phases)
        }
    })
}

pub fn next_change(
    profile: &ChargingProfile,
    at: u64,
    transaction_start: Option<u64>,
) -> Option<u64> {
    let schedule = &profile.charging_schedule;
    let mut instants = vec![
        profile.valid_from.map(|from| from.unix_ms),
        profile.valid_to.map(|to| to.unix_ms),
        first_start(profile),
    ];
    if let Some(start) = cycle_start(profile, at, transaction_start) {
        let periods = schedule.charging_schedule_period.iter();
        instants.extend(periods.map(|period| Some(start + ms(period.start_period))));
        instants.push(schedule.duration.map(|seconds| start + ms(seconds)));
        instants.push(cycle_ms(profile).map(|cycle| start + cycle));
    }
    instants
        .into_iter()
        .flatten()
        .filter(|&instant| instant > at)
        .min()
}

fn valid_at(profile: &ChargingProfile, at: u64) -> bool {
    profile.valid_from.is_none_or(|from| from.unix_ms <= at)
        && profile.valid_to.is_none_or(|to| at < to.unix_ms)
}

fn cycle_start(profile: &ChargingProfile, at: u64, transaction_start: Option<u64>) -> Option<u64> {
    let start_schedule = first_start(profile);
    match profile.charging_profile_kind {
        ChargingProfileKind::Absolute => Some(start_schedule.or(transaction_start).unwrap_or(0)),
        ChargingProfileKind::Relative => transaction_start,
        ChargingProfileKind::Recurring => {
            let first = start_schedule?;
            let cycle = cycle_ms(profile)?;
            match at.checked_sub(first) {
                Some(elapsed) => Some(first + elapsed / cycle * cycle),
                None => Some(first),
            }
        }
    }
}

fn first_start(profile: &ChargingProfile) -> Option<u64> {
    let start = profile.charging_schedule.start_schedule?;
    Some(start.unix_ms)
}

fn cycle_ms(profile: &ChargingProfile) -> Option<u64> {
    if profile.charging_profile_kind != ChargingProfileKind::Recurring {
        return None;
    }
    match profile.recurrency_kind? {
        RecurrencyKind::Daily => Some(DAY_MS),
        RecurrencyKind::Weekly => Some(7 * DAY_MS),
    }
}

fn ms(seconds: u32) -> u64 {
    u64::from(seconds) * 1000
}
