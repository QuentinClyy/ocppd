use crate::api::intent::EvSpec;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn profile(id: i32, purpose: &str, stack_level: u32, schedule: Value) -> Value {
    json!({
        "chargingProfileId": id,
        "stackLevel": stack_level,
        "chargingProfilePurpose": purpose,
        "chargingProfileKind": "Absolute",
        "chargingSchedule": schedule,
    })
}

fn amps(periods: &[(u32, f64)]) -> Value {
    let periods: Vec<_> = periods
        .iter()
        .map(|&(start, limit)| json!({"startPeriod": start, "limit": limit}))
        .collect();
    json!({"chargingRateUnit": "A", "chargingSchedulePeriod": periods})
}

fn set(csms: &mut Csms, connector: u32, profile: Value) {
    let request = json!({"connectorId": connector, "csChargingProfiles": profile});
    match csms.call(0, "SetChargingProfile", request) {
        Frame::CallResult { payload, .. } => assert_eq!(payload["status"], "Accepted"),
        other => panic!("expected a result, got {other:?}"),
    }
}

fn composite(csms: &mut Csms, now: u64, connector: u32, duration: u32, unit: &str) -> Value {
    let request = json!({"connectorId": connector, "duration": duration, "chargingRateUnit": unit});
    match csms.call(now, "GetCompositeSchedule", request) {
        Frame::CallResult { payload, .. } => payload,
        other => panic!("expected a result, got {other:?}"),
    }
}

fn periods(csms: &mut Csms, now: u64, connector: u32, duration: u32) -> Vec<(u64, f64)> {
    let payload = composite(csms, now, connector, duration, "A");
    assert_eq!(payload["status"], "Accepted");
    let periods = payload["chargingSchedule"]["chargingSchedulePeriod"]
        .as_array()
        .unwrap()
        .clone();
    periods
        .iter()
        .map(|period| {
            (
                period["startPeriod"].as_u64().unwrap(),
                period["limit"].as_f64().unwrap(),
            )
        })
        .collect()
}

fn charging_with_default() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(
        &mut csms,
        0,
        profile(1, "TxDefaultProfile", 0, amps(&[(0, 10.0)])),
    );
    csms
}

fn recurring(kind: &str) -> Value {
    let mut schedule = amps(&[(0, 16.0), (3600, 8.0)]);
    schedule["startSchedule"] = json!("1970-01-01T00:00:00Z");
    let mut recurring = profile(1, "ChargePointMaxProfile", 0, schedule);
    recurring["chargingProfileKind"] = json!("Recurring");
    recurring["recurrencyKind"] = json!(kind);
    recurring
}

#[test]
fn given_no_profiles_when_composite_schedule_requested_then_the_hardware_limit() {
    let mut csms = Csms::booted();

    let payload = composite(&mut csms, 0, 1, 3600, "W");

    assert_eq!(payload["connectorId"], 1);
    assert_eq!(payload["scheduleStart"], "1970-01-01T00:00:00Z");
    assert_eq!(
        payload["chargingSchedule"]["chargingSchedulePeriod"],
        json!([{"startPeriod": 0, "limit": 22080.0}])
    );
}

#[test]
fn given_charge_point_max_profile_when_composite_schedules_requested_then_every_connector_capped() {
    let mut csms = Csms::booted();
    set(
        &mut csms,
        0,
        profile(1, "ChargePointMaxProfile", 0, amps(&[(0, 16.0)])),
    );

    let schedules = [
        periods(&mut csms, 0, 0, 3600),
        periods(&mut csms, 0, 1, 3600),
    ];

    assert_eq!(schedules, [vec![(0, 16.0)], vec![(0, 16.0)]]);
}

#[test]
fn given_tx_default_profile_without_transaction_when_composite_schedule_requested_then_not_applied()
{
    let mut csms = Csms::booted();
    set(
        &mut csms,
        0,
        profile(1, "TxDefaultProfile", 0, amps(&[(0, 10.0)])),
    );

    let schedule = periods(&mut csms, 0, 1, 3600);

    assert_eq!(schedule, vec![(0, 32.0)]);
}

#[test]
fn given_tx_default_profile_during_a_transaction_when_composite_schedule_requested_then_applied() {
    let mut csms = charging_with_default();

    let schedule = periods(&mut csms, 0, 1, 3600);

    assert_eq!(schedule, vec![(0, 10.0)]);
}

#[test]
fn given_tx_default_and_tx_profiles_when_composite_schedule_requested_then_the_tx_profile_wins() {
    let mut csms = charging_with_default();
    set(&mut csms, 1, profile(2, "TxProfile", 0, amps(&[(0, 20.0)])));

    let schedule = periods(&mut csms, 0, 1, 3600);

    assert_eq!(schedule, vec![(0, 20.0)]);
}

#[test]
fn given_tx_profile_above_the_charge_point_max_when_composite_schedule_requested_then_the_max_caps_it()
 {
    let mut csms = charging_with_default();
    set(&mut csms, 1, profile(2, "TxProfile", 0, amps(&[(0, 20.0)])));
    set(
        &mut csms,
        0,
        profile(3, "ChargePointMaxProfile", 0, amps(&[(0, 16.0)])),
    );

    let schedule = periods(&mut csms, 0, 1, 3600);

    assert_eq!(schedule, vec![(0, 16.0)]);
}

#[test]
fn given_short_higher_stack_level_when_composite_schedule_requested_then_it_wins_until_its_schedule_ends()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(
        &mut csms,
        1,
        profile(1, "TxDefaultProfile", 0, amps(&[(0, 10.0)])),
    );
    let mut short = amps(&[(0, 20.0)]);
    short["duration"] = json!(600);
    set(&mut csms, 1, profile(2, "TxDefaultProfile", 1, short));

    let schedule = periods(&mut csms, 0, 1, 3600);

    assert_eq!(schedule, vec![(0, 20.0), (600, 10.0)]);
}

#[test]
fn given_relative_profile_when_composite_schedule_requested_later_then_counted_from_the_transaction_start()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    let mut relative = profile(1, "TxProfile", 0, amps(&[(0, 16.0), (120, 8.0)]));
    relative["chargingProfileKind"] = json!("Relative");
    set(&mut csms, 1, relative);

    let schedule = periods(&mut csms, MINUTE, 1, 3600);

    assert_eq!(schedule, vec![(0, 16.0), (60, 8.0)]);
}

#[test]
fn given_daily_profile_when_two_days_requested_then_repeated_daily() {
    let mut csms = Csms::booted();
    set(&mut csms, 0, recurring("Daily"));

    let schedule = periods(&mut csms, 0, 0, 2 * 86_400);

    assert_eq!(
        schedule,
        vec![(0, 16.0), (3600, 8.0), (86_400, 16.0), (90_000, 8.0)]
    );
}

#[test]
fn given_profile_valid_to_half_an_hour_when_an_hour_requested_then_it_ends_at_valid_to() {
    let mut csms = Csms::booted();
    let mut max = profile(1, "ChargePointMaxProfile", 0, amps(&[(0, 16.0)]));
    max["validTo"] = json!("1970-01-01T00:30:00Z");
    set(&mut csms, 0, max);

    let schedule = periods(&mut csms, 0, 0, 3600);

    assert_eq!(schedule, vec![(0, 16.0), (1800, 32.0)]);
}

#[test]
fn given_one_phase_amp_limit_when_requested_in_watts_then_only_that_phase_counted() {
    let mut csms = Csms::booted();
    let schedule = json!({
        "chargingRateUnit": "A",
        "chargingSchedulePeriod": [{"startPeriod": 0, "limit": 16.0, "numberPhases": 1}],
    });
    set(
        &mut csms,
        0,
        profile(1, "ChargePointMaxProfile", 0, schedule),
    );

    let payload = composite(&mut csms, 0, 0, 3600, "W");

    let period = &payload["chargingSchedule"]["chargingSchedulePeriod"][0];
    assert_eq!(period["limit"], 3680.0);
}

#[test]
fn given_one_connector_when_composite_schedule_of_connector_2_requested_then_rejected() {
    let mut csms = Csms::booted();

    let payload = composite(&mut csms, 0, 2, 3600, "W");

    assert_eq!(payload, json!({"status": "Rejected"}));
}
