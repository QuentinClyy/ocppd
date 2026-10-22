use crate::fixtures::v201::charging_profiles::{
    AMPS_TO_W, HARDWARE_W, absolute, charging, entries, offered_w, profile, reason_code, set,
    tx_profile,
};
use crate::fixtures::v201::station::Run;
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

fn default() -> Value {
    profile(1, "TxDefaultProfile", &[(0, 8.0), (60, 6.0)])
}

fn invalid_schedules() -> Vec<(&'static str, Value)> {
    let mut cases = Vec::new();
    cases.push(("K01.FR.31", profile(1, "TxDefaultProfile", &[(10, 8.0)])));
    cases.push((
        "K01.FR.35",
        profile(1, "TxDefaultProfile", &[(0, 8.0), (60, 6.0), (30, 7.0)]),
    ));
    let mut two = default();
    let schedule = two["chargingSchedule"][0].clone();
    two["chargingSchedule"]
        .as_array_mut()
        .unwrap()
        .push(schedule);
    cases.push(("K01.FR.34", two));
    let mut no_start = absolute(default());
    no_start["chargingSchedule"][0]
        .as_object_mut()
        .unwrap()
        .remove("startSchedule");
    cases.push(("K01.FR.40", no_start));
    let mut started = default();
    started["chargingSchedule"][0]["startSchedule"] = json!("1970-01-01T00:00:00Z");
    cases.push(("K01.FR.41", started));
    let mut phase = default();
    phase["chargingSchedule"][0]["chargingSchedulePeriod"][0]["numberPhases"] = json!(1);
    phase["chargingSchedule"][0]["chargingSchedulePeriod"][0]["phaseToUse"] = json!(1);
    cases.push(("K01.FR.48", phase));
    let mut recurring = absolute(default());
    recurring["chargingProfileKind"] = json!("Recurring");
    cases.push(("K01.FR.60", recurring));
    let mut stacked = default();
    stacked["stackLevel"] = json!(11);
    cases.push(("ProfileStackLevel", stacked));
    let periods: Vec<(u32, f64)> = (0..25).map(|minute| (minute * 60, 8.0)).collect();
    cases.push((
        "PeriodsPerSchedule",
        profile(1, "TxDefaultProfile", &periods),
    ));
    cases
}

#[test]
fn given_charging_ev_when_tx_profile_set_then_accepted_and_limits_its_transaction_per_k01_fr_04_and_tc_k_60()
 {
    let mut run = Run::accepted(86_400);
    let transaction_id = charging(&mut run, 0, 1);

    let response = set(&mut run, 0, 1, tx_profile(1, &transaction_id, &[(0, 8.0)]));

    assert_eq!(response, json!({"status": "Accepted"}));
    assert_eq!(offered_w(&run, 0), Some(8.0 * AMPS_TO_W));
    assert!(run.events.contains(&EventKind::ChargingLimitChanged {
        evse: 1,
        limit_w: 8.0 * AMPS_TO_W
    }));
    assert_eq!(entries(&mut run), "1");
}

#[test]
fn given_charging_ev_without_profile_when_offered_power_read_then_hardware_limit() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0, 1);

    let offered = offered_w(&run, 0);

    assert_eq!(offered, Some(HARDWARE_W));
}

#[test]
fn given_no_transaction_when_tx_profile_without_transaction_id_set_then_tx_not_found_per_k01_fr_09_and_tc_k_02()
 {
    let mut run = Run::accepted(86_400);

    let reason = reason_code(&mut run, 1, profile(1, "TxProfile", &[(0, 8.0)]));

    assert_eq!(reason, "TxNotFound");
}

#[test]
fn given_charging_ev_when_tx_profile_for_another_transaction_set_then_tx_not_found_and_not_applied_per_k01_fr_33_and_tc_k_16()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0, 1);

    let reason = reason_code(&mut run, 1, tx_profile(1, "other", &[(0, 8.0)]));

    assert_eq!(reason, "TxNotFound");
    assert_eq!(offered_w(&run, 0), Some(HARDWARE_W));
}

#[test]
fn given_charging_ev_when_tx_profile_without_transaction_id_set_then_tx_not_found_per_k01_fr_03() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0, 1);

    let reason = reason_code(&mut run, 1, profile(1, "TxProfile", &[(0, 8.0)]));

    assert_eq!(reason, "TxNotFound");
}

#[test]
fn given_charging_ev_when_tx_profile_set_on_evse_0_then_invalid_profile_per_k01_fr_16_and_tc_k_11()
{
    let mut run = Run::accepted(86_400);
    let transaction_id = charging(&mut run, 0, 1);

    let reason = reason_code(&mut run, 0, tx_profile(1, &transaction_id, &[(0, 8.0)]));

    assert_eq!(reason, "InvalidProfile");
}

#[test]
fn given_one_evse_when_profile_set_on_evse_2_then_unknown_evse_per_k01_fr_28_and_tc_k_14() {
    let mut run = Run::accepted(86_400);

    let reason = reason_code(&mut run, 2, profile(1, "TxDefaultProfile", &[(0, 8.0)]));

    assert_eq!(reason, "UnknownEVSE");
}

#[test]
fn given_booted_station_when_external_constraints_set_then_invalid_profile_per_k01_fr_22() {
    let mut run = Run::accepted(86_400);
    let external = absolute(profile(
        1,
        "ChargingStationExternalConstraints",
        &[(0, 8.0)],
    ));

    let reason = reason_code(&mut run, 0, external);

    assert_eq!(reason, "InvalidProfile");
}

#[test]
fn given_booted_station_when_station_max_set_on_an_evse_then_invalid_profile_per_k04_fr_03() {
    let mut run = Run::accepted(86_400);
    let max = absolute(profile(1, "ChargingStationMaxProfile", &[(0, 8.0)]));

    let reason = reason_code(&mut run, 1, max);

    assert_eq!(reason, "InvalidProfile");
}

#[test]
fn given_booted_station_when_relative_station_max_set_then_invalid_profile_per_k01_fr_38() {
    let mut run = Run::accepted(86_400);
    let relative = profile(1, "ChargingStationMaxProfile", &[(0, 8.0)]);

    let reason = reason_code(&mut run, 0, relative);

    assert_eq!(reason, "InvalidProfile");
}

#[test]
fn given_booted_station_when_absolute_station_max_set_on_evse_0_then_accepted_per_tc_k_03() {
    let mut run = Run::accepted(86_400);
    let max = absolute(profile(1, "ChargingStationMaxProfile", &[(0, 8.0)]));

    let response = set(&mut run, 0, 0, max);

    assert_eq!(response["status"], "Accepted");
}

#[test]
fn given_booted_station_when_invalid_schedules_set_then_each_invalid_schedule_per_k01_fr_31_fr_34_fr_35_fr_40_fr_41_fr_48_and_fr_60()
 {
    let mut run = Run::accepted(86_400);
    let cases = invalid_schedules();

    let reasons: Vec<_> = cases
        .iter()
        .map(|(_, profile)| reason_code(&mut run, 1, profile.clone()))
        .collect();

    for ((requirement, _), reason) in cases.iter().zip(&reasons) {
        assert_eq!(reason, "InvalidSchedule", "{requirement}");
    }
}

#[test]
fn given_booted_station_when_valid_default_set_then_accepted_per_tc_k_01() {
    let mut run = Run::accepted(86_400);

    let response = set(&mut run, 0, 1, default());

    assert_eq!(response["status"], "Accepted");
}

#[test]
fn given_default_on_evse_0_when_default_on_an_evse_at_the_same_stack_level_then_duplicate_per_k01_fr_53()
 {
    let mut run = Run::accepted(86_400);
    set(&mut run, 0, 0, profile(1, "TxDefaultProfile", &[(0, 8.0)]));

    let reason = reason_code(&mut run, 1, profile(2, "TxDefaultProfile", &[(0, 8.0)]));

    assert_eq!(reason, "DuplicateProfile");
}

#[test]
fn given_default_on_an_evse_when_default_on_evse_0_at_the_same_stack_level_then_duplicate_per_k01_fr_52()
 {
    let mut run = Run::accepted(86_400);
    set(&mut run, 0, 1, profile(1, "TxDefaultProfile", &[(0, 8.0)]));

    let reason = reason_code(&mut run, 0, profile(2, "TxDefaultProfile", &[(0, 8.0)]));

    assert_eq!(reason, "DuplicateProfile");
}
