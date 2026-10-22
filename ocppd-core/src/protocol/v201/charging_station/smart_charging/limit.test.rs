use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::charging_profiles::{
    AMPS_TO_W, HALF_MINUTE, HARDWARE_W, absolute, charging, entries, offered_w, profile, set,
    tx_profile,
};
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::json;

fn default_then_charging() -> (Run, String) {
    let mut run = Run::accepted(86_400);
    set(&mut run, 0, 1, profile(1, "TxDefaultProfile", &[(0, 10.0)]));
    let transaction_id = charging(&mut run, 0, 1);
    (run, transaction_id)
}

fn two_evses_with_shared_max() -> Run {
    let mut run = Run::accepted_with(2, 86_400);
    let max = absolute(profile(1, "ChargingStationMaxProfile", &[(0, 20.0)]));
    set(&mut run, 0, 0, max);
    run
}

#[test]
fn given_charging_ev_with_default_when_same_profile_id_set_again_then_replaced_per_k01_fr_05_and_tc_k_04()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0, 1);
    set(&mut run, 0, 1, profile(1, "TxDefaultProfile", &[(0, 8.0)]));
    let mut moved = profile(1, "TxDefaultProfile", &[(0, 6.0)]);
    moved["stackLevel"] = json!(3);

    set(&mut run, 0, 1, moved);

    assert_eq!(entries(&mut run), "1");
    assert_eq!(offered_w(&run, 0), Some(6.0 * AMPS_TO_W));
}

#[test]
fn given_two_charging_evs_when_default_set_on_evse_0_then_both_limited_without_a_copy_per_k01_fr_14_and_fr_32_and_tc_k_10()
 {
    let mut run = Run::accepted_with(2, 86_400);
    charging(&mut run, 0, 1);
    charging(&mut run, 0, 2);

    set(&mut run, 0, 0, profile(1, "TxDefaultProfile", &[(0, 10.0)]));

    assert_eq!(offered_w(&run, 0), Some(10.0 * AMPS_TO_W));
    assert_eq!(offered_w(&run, 1), Some(10.0 * AMPS_TO_W));
    assert_eq!(entries(&mut run), "1", "applied, not copied");
}

#[test]
fn given_default_when_ev_starts_charging_then_limited_by_the_default_per_k02_fr_04() {
    let mut run = Run::accepted(86_400);
    set(&mut run, 0, 1, profile(1, "TxDefaultProfile", &[(0, 10.0)]));

    charging(&mut run, 0, 1);

    assert_eq!(offered_w(&run, 0), Some(10.0 * AMPS_TO_W));
}

#[test]
fn given_default_limited_transaction_when_tx_profile_set_then_it_overrules_the_default_per_k02_fr_04()
 {
    let (mut run, transaction_id) = default_then_charging();

    set(&mut run, 0, 1, tx_profile(2, &transaction_id, &[(0, 16.0)]));

    assert_eq!(offered_w(&run, 0), Some(16.0 * AMPS_TO_W));
}

#[test]
fn given_tx_profile_when_its_transaction_ends_then_deleted_per_k02_fr_05() {
    let (mut run, transaction_id) = default_then_charging();
    set(&mut run, 0, 1, tx_profile(2, &transaction_id, &[(0, 16.0)]));

    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(MINUTE);

    assert_eq!(entries(&mut run), "1");
}

#[test]
fn given_short_tx_profile_when_set_then_its_end_scheduled_per_k02_fr_08() {
    let (mut run, transaction_id) = default_then_charging();
    let mut short = tx_profile(2, &transaction_id, &[(0, 16.0)]);
    short["chargingSchedule"][0]["duration"] = json!(30);

    set(&mut run, 0, 1, short);

    assert_eq!(run.station.next_deadline(), Some(HALF_MINUTE));
}

#[test]
fn given_short_tx_profile_when_it_expires_then_falls_back_to_the_default_per_k02_fr_08() {
    let (mut run, transaction_id) = default_then_charging();
    let mut short = tx_profile(2, &transaction_id, &[(0, 16.0)]);
    short["chargingSchedule"][0]["duration"] = json!(30);
    set(&mut run, 0, 1, short);

    run.input(HALF_MINUTE, Input::Tick);

    assert_eq!(offered_w(&run, 0), Some(10.0 * AMPS_TO_W));
}

#[test]
fn given_charging_ev_when_stepped_tx_profile_set_then_wakes_up_at_the_step_per_k01_fr_12() {
    let mut run = Run::accepted(86_400);
    let transaction_id = charging(&mut run, 0, 1);

    set(
        &mut run,
        0,
        1,
        tx_profile(1, &transaction_id, &[(0, 16.0), (30, 6.0)]),
    );

    assert_eq!(run.station.next_deadline(), Some(HALF_MINUTE));
}

#[test]
fn given_stepped_tx_profile_when_the_step_comes_then_new_limit_applied_per_k01_fr_12() {
    let mut run = Run::accepted(86_400);
    let transaction_id = charging(&mut run, 0, 1);
    set(
        &mut run,
        0,
        1,
        tx_profile(1, &transaction_id, &[(0, 16.0), (30, 6.0)]),
    );

    run.input(HALF_MINUTE, Input::Tick);

    assert_eq!(offered_w(&run, 0), Some(6.0 * AMPS_TO_W));
}

#[test]
fn given_station_max_when_one_ev_charges_then_it_gets_the_whole_max_per_k04_fr_05() {
    let mut run = two_evses_with_shared_max();

    charging(&mut run, 0, 1);

    assert_eq!(offered_w(&run, 0), Some(20.0 * AMPS_TO_W));
}

#[test]
fn given_station_max_and_one_charging_ev_when_a_second_ev_charges_then_the_max_is_shared_per_k04_fr_05()
 {
    let mut run = two_evses_with_shared_max();
    charging(&mut run, 0, 1);

    charging(&mut run, 0, 2);

    assert_eq!(offered_w(&run, 0), Some(10.0 * AMPS_TO_W));
    assert_eq!(offered_w(&run, 1), Some(10.0 * AMPS_TO_W));
}

#[test]
fn given_charging_ev_when_future_station_max_set_then_not_yet_applied_and_its_start_scheduled_per_k01_fr_30_and_tc_k_23()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0, 1);
    let mut later = absolute(profile(1, "ChargingStationMaxProfile", &[(0, 8.0)]));
    later["chargingSchedule"][0]["startSchedule"] = json!("1970-01-01T00:00:30Z");

    set(&mut run, 0, 0, later);

    assert_eq!(offered_w(&run, 0), Some(HARDWARE_W));
    assert_eq!(run.station.next_deadline(), Some(HALF_MINUTE));
}

#[test]
fn given_future_station_max_when_its_start_comes_then_applied_per_k01_fr_30() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0, 1);
    let mut later = absolute(profile(1, "ChargingStationMaxProfile", &[(0, 8.0)]));
    later["chargingSchedule"][0]["startSchedule"] = json!("1970-01-01T00:00:30Z");
    set(&mut run, 0, 0, later);

    run.input(HALF_MINUTE, Input::Tick);

    assert_eq!(offered_w(&run, 0), Some(8.0 * AMPS_TO_W));
}
