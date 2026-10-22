use crate::fixtures::v201::charging_profiles::{
    HARDWARE_W, absolute, charging, profile, set, tx_profile,
};
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn composite(run: &mut Run, now: u64, evse: u32, unit: &str) -> Value {
    let request = json!({"evseId": evse, "duration": 3600, "chargingRateUnit": unit});
    run.result(now, "GetCompositeSchedule", request)
}

fn periods(run: &mut Run, now: u64, evse: u32, unit: &str) -> Vec<(u64, f64)> {
    let response = composite(run, now, evse, unit);
    assert_eq!(response["status"], "Accepted");
    let periods = response["schedule"]["chargingSchedulePeriod"]
        .as_array()
        .unwrap()
        .clone();
    periods
        .iter()
        .map(|period| {
            let start = period["startPeriod"].as_u64().unwrap();
            (start, period["limit"].as_f64().unwrap())
        })
        .collect()
}

#[test]
fn given_no_profiles_when_composite_asked_then_hardware_limit_reported_per_k08_fr_02_and_tc_k_39() {
    let mut run = Run::accepted(86_400);

    let response = composite(&mut run, 0, 1, "W");

    assert_eq!(
        response["schedule"],
        json!({
            "chargingSchedulePeriod": [{"startPeriod": 0, "limit": HARDWARE_W}],
            "evseId": 1,
            "duration": 3600,
            "scheduleStart": "1970-01-01T00:00:00Z",
            "chargingRateUnit": "W",
        })
    );
}

#[test]
fn given_one_evse_when_composite_of_evse_2_asked_then_rejected_per_k08_fr_05_and_tc_k_47() {
    let mut run = Run::accepted(86_400);

    let response = composite(&mut run, 0, 2, "W");

    assert_eq!(response, json!({"status": "Rejected"}));
}

#[test]
fn given_idle_evse_with_default_profile_when_composite_asked_then_default_as_if_charging_now_per_k08_fr_06()
 {
    let mut run = Run::accepted(86_400);
    let default = profile(1, "TxDefaultProfile", &[(0, 10.0), (600, 20.0)]);
    set(&mut run, 0, 1, default);

    let periods = periods(&mut run, 60_000, 1, "A");

    assert_eq!(periods, vec![(0, 10.0), (600, 20.0)]);
}

#[test]
fn given_default_and_station_max_profiles_when_composite_asked_then_lowest_limit_at_every_moment_per_k08_fr_04()
 {
    let mut run = Run::accepted(86_400);
    let default = profile(1, "TxDefaultProfile", &[(0, 10.0), (600, 20.0)]);
    set(&mut run, 0, 1, default);
    let max = absolute(profile(2, "ChargingStationMaxProfile", &[(0, 16.0)]));
    set(&mut run, 0, 0, max);

    let periods = periods(&mut run, 0, 1, "A");

    assert_eq!(periods, vec![(0, 10.0), (600, 16.0)]);
}

#[test]
fn given_tx_profile_on_a_running_transaction_when_composite_asked_later_then_schedule_counted_from_its_start()
 {
    let mut run = Run::accepted(86_400);
    let transaction_id = charging(&mut run, 0, 1);
    let tx = tx_profile(1, &transaction_id, &[(0, 16.0), (600, 8.0)]);
    set(&mut run, 0, 1, tx);

    let periods = periods(&mut run, 60_000, 1, "A");

    assert_eq!(periods, vec![(0, 16.0), (540, 8.0)]);
}

#[test]
fn given_two_evses_without_profiles_when_composite_of_evse_0_asked_then_whole_grid_connection_per_k08_fr_03()
 {
    let mut run = Run::accepted_with(2, 86_400);

    let periods = periods(&mut run, 0, 0, "W");

    assert_eq!(periods, vec![(0, 2.0 * HARDWARE_W)]);
}

#[test]
fn given_station_max_profile_when_composite_of_evse_0_asked_then_its_steps_per_k08_fr_03() {
    let mut run = Run::accepted_with(2, 86_400);
    let steps = [(0, 30_000.0), (600, 20_000.0)];
    let mut max = absolute(profile(1, "ChargingStationMaxProfile", &steps));
    max["chargingSchedule"][0]["chargingRateUnit"] = json!("W");
    set(&mut run, 0, 0, max);

    let periods = periods(&mut run, 0, 0, "W");

    assert_eq!(periods, vec![(0, 30_000.0), (600, 20_000.0)]);
}

#[test]
fn given_default_in_watts_when_composite_asked_in_amps_then_rounded_down_to_a_tenth() {
    let mut run = Run::accepted(86_400);
    let mut watts = profile(1, "TxDefaultProfile", &[(0, 5010.0)]);
    watts["chargingSchedule"][0]["chargingRateUnit"] = json!("W");
    set(&mut run, 0, 1, watts);

    let periods = periods(&mut run, 0, 1, "A");

    assert_eq!(periods, vec![(0, 7.2)]);
}
