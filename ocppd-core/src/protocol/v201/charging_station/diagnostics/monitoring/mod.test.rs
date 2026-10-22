use crate::fixtures::v201::monitors::{data, events, monitor, set_heartbeat, status};
use crate::fixtures::v201::station::Run;
use crate::wire::frame::Frame;
use serde_json::json;

fn upper_600() -> Run {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("UpperThreshold", 600.0, 5));
    run
}

fn error_code(reply: Frame) -> String {
    match reply {
        Frame::CallError { code, .. } => code,
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn given_booted_station_when_upper_threshold_set_then_accepted_with_id_1_per_n04_fr_11_and_tc_n_08()
{
    let mut run = Run::accepted(300);

    let result = monitor(&mut run, data("UpperThreshold", 600.0, 5));

    assert_eq!(status(&result), "Accepted");
    assert_eq!(result["id"], 1);
}

#[test]
fn given_monitor_when_its_id_set_on_another_variable_then_rejected_per_n04_fr_16() {
    let mut run = upper_600();
    let mut moved = data("Delta", 1.0, 5);
    moved["id"] = json!(1);
    moved["variable"] = json!({"name": "OfflineThreshold"});

    let result = monitor(&mut run, moved);

    assert_eq!(status(&result), "Rejected");
}

#[test]
fn given_monitor_when_its_id_set_on_the_same_variable_then_replaced_per_n04_fr_12_and_tc_n_40() {
    let mut run = upper_600();
    let mut higher = data("UpperThreshold", 1000.0, 5);
    higher["id"] = json!(1);

    let result = monitor(&mut run, higher);

    assert_eq!(status(&result), "Accepted");
}

#[test]
fn given_replaced_threshold_when_value_exceeds_the_old_one_only_then_no_event_per_n04_fr_12() {
    let mut run = upper_600();
    let mut higher = data("UpperThreshold", 1000.0, 5);
    higher["id"] = json!(1);
    monitor(&mut run, higher);

    set_heartbeat(&mut run, "900");

    assert!(events(&run).is_empty(), "the new threshold applies");
}

#[test]
fn given_monitor_1_when_monitors_1_and_7_cleared_then_accepted_and_not_found_per_n06_fr_01_fr_02_and_tc_n_18()
 {
    let mut run = upper_600();

    let response = run.result(0, "ClearVariableMonitoring", json!({"id": [1, 7]}));

    assert_eq!(
        response["clearMonitoringResult"],
        json!([{"status": "Accepted", "id": 1}, {"status": "NotFound", "id": 7}])
    );
}

#[test]
fn given_booted_station_when_51_monitors_set_then_occurrence_violation_per_n04_fr_09() {
    let mut run = Run::accepted(300);
    let request = json!({"setMonitoringData": vec![data("Delta", 1.0, 5); 51]});

    let reply = run.csms(0, "SetVariableMonitoring", request);

    assert_eq!(error_code(reply), "OccurrenceConstraintViolation");
}

#[test]
fn given_booted_station_when_51_monitors_cleared_then_occurrence_violation_per_n06_fr_06() {
    let mut run = Run::accepted(300);
    let clear = json!({"id": (0..51).collect::<Vec<i32>>()});

    let reply = run.csms(0, "ClearVariableMonitoring", clear);

    assert_eq!(error_code(reply), "OccurrenceConstraintViolation");
}
