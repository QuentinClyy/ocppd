use crate::api::io::Input;
use crate::fixtures::v201::monitors::{data, events, monitor, set_heartbeat, status};
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn upper_600() -> Run {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("UpperThreshold", 600.0, 5));
    run
}

fn invalid_monitors() -> Vec<(&'static str, Value, &'static str)> {
    let mut unknown_component = data("Delta", 1.0, 5);
    unknown_component["component"] = json!({"name": "Nowhere"});
    let mut unknown_variable = data("Delta", 1.0, 5);
    unknown_variable["variable"] = json!({"name": "Nothing"});
    let mut boolean = data("UpperThreshold", 1.0, 5);
    boolean["component"] = json!({"name": "TxCtrlr"});
    boolean["variable"] = json!({"name": "StopTxOnInvalidId"});
    let mut unknown_id = data("Delta", 1.0, 5);
    unknown_id["id"] = json!(42);
    vec![
        ("N04.FR.03", unknown_component, "UnknownComponent"),
        ("N04.FR.04", unknown_variable, "UnknownVariable"),
        ("N04.FR.05", boolean, "UnsupportedMonitorType"),
        ("period under 1 s", data("Periodic", 0.5, 5), "Rejected"),
        ("N04.FR.06", data("LowerThreshold", -1.0, 5), "Rejected"),
        ("N04.FR.10", data("UpperThreshold", 900.0, 5), "Duplicate"),
        ("N04.FR.13", unknown_id, "Rejected"),
        ("N04.FR.14", data("Delta", -5.0, 5), "Rejected"),
        ("N04.FR.20", data("Delta", 0.0, 5), "Rejected"),
        ("severity", data("Delta", 5.0, 10), "Rejected"),
    ]
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
fn given_upper_threshold_when_invalid_monitors_set_then_each_refused_in_order_per_n04_fr_01_to_fr_06_fr_10_fr_13_fr_14_and_fr_20()
 {
    let mut run = upper_600();
    let cases = invalid_monitors();
    let request: Vec<Value> = cases.iter().map(|(_, data, _)| data.clone()).collect();

    let response = run.result(
        0,
        "SetVariableMonitoring",
        json!({"setMonitoringData": request}),
    );

    let results = response["setMonitoringResult"].as_array().unwrap();
    assert_eq!(results.len(), cases.len(), "N04.FR.01");
    for ((requirement, data, expected), result) in cases.iter().zip(results) {
        assert_eq!(status(result), *expected, "{requirement}");
        assert_eq!(result["component"], data["component"], "N04.FR.02");
        assert_eq!(result["variable"], data["variable"], "N04.FR.02");
    }
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

fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

fn level(run: &mut Run, severity: i32) -> serde_json::Value {
    let request = json!({"severity": severity});
    run.result(0, "SetMonitoringLevel", request)["status"].clone()
}

fn clock_aligned(run: &mut Run) {
    let request = json!({"setMonitoringData": [data("PeriodicClockAligned", 900.0, 5)]});
    run.result(100_000, "SetVariableMonitoring", request);
}

#[test]
fn given_booted_station_when_periodic_monitor_set_then_first_report_due_after_its_period_per_n08_fr_04()
 {
    let mut run = Run::accepted(300);

    monitor(&mut run, data("Periodic", 60.0, 5));

    assert_eq!(run.station.next_deadline(), Some(MINUTE));
}

#[test]
fn given_periodic_monitor_when_its_period_passes_then_reports_the_value_and_reschedules_per_n08_fr_05_and_fr_06_and_tc_n_24()
 {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("Periodic", 60.0, 5));

    tick(&mut run, MINUTE);

    let events = events(&run);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["trigger"], "Periodic", "N08.FR.05");
    assert_eq!(events[0]["actualValue"], "300");
    assert_eq!(run.station.next_deadline(), Some(2 * MINUTE));
}

#[test]
fn given_booted_station_when_clock_aligned_monitor_set_then_first_report_due_on_the_clock_per_n08_fr_07()
 {
    let mut run = Run::accepted(86_400);

    clock_aligned(&mut run);

    assert_eq!(run.station.next_deadline(), Some(15 * MINUTE));
}

#[test]
fn given_clock_aligned_monitor_when_the_clock_boundary_passes_then_reports_and_reschedules_per_n08_fr_07()
 {
    let mut run = Run::accepted(86_400);
    clock_aligned(&mut run);

    tick(&mut run, 15 * MINUTE);

    assert_eq!(events(&run).len(), 1);
    assert_eq!(run.station.next_deadline(), Some(30 * MINUTE));
}

#[test]
fn given_transaction_only_monitor_without_transaction_when_its_period_passes_then_no_report_and_waits_for_the_next()
 {
    let mut run = Run::accepted(300);
    let mut during = data("Periodic", 60.0, 5);
    during["transaction"] = json!(true);
    monitor(&mut run, during);

    tick(&mut run, MINUTE);

    assert!(events(&run).is_empty());
    assert_eq!(
        run.station.next_deadline(),
        Some(2 * MINUTE),
        "no busy wake-up"
    );
}

#[test]
fn given_booted_station_when_monitoring_level_10_set_then_rejected_per_n05_fr_02() {
    let mut run = Run::accepted(300);

    let status = level(&mut run, 10);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_station_when_monitoring_level_minus_1_set_then_rejected_per_n05_fr_02_and_tc_n_17()
{
    let mut run = Run::accepted(300);

    let status = level(&mut run, -1);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_station_when_monitoring_level_3_set_then_accepted_and_reported_per_n05_fr_01() {
    let mut run = Run::accepted(300);

    let status = level(&mut run, 3);

    assert_eq!(status, "Accepted");
    let get = json!({"getVariableData": [{"component": {"name": "MonitoringCtrlr"}, "variable": {"name": "ActiveMonitoringLevel"}}]});
    let response = run.result(0, "GetVariables", get);
    assert_eq!(response["getVariableResult"][0]["attributeValue"], "3");
}

#[test]
fn given_level_3_and_monitors_of_severity_5_and_2_when_both_exceeded_then_only_severity_2_reported_per_n05_fr_03_and_n07_fr_15()
 {
    let mut run = Run::accepted(300);
    level(&mut run, 3);
    let accepted = monitor(&mut run, data("UpperThreshold", 600.0, 5));
    monitor(&mut run, data("UpperThreshold", 700.0, 2));

    set_heartbeat(&mut run, "900");

    assert_eq!(status(&accepted), "Accepted");
    let events = events(&run);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["variableMonitoringId"], 2);
}
