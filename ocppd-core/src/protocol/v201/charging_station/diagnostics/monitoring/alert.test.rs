use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::monitors::{data, events, monitor, set_heartbeat, status};
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::fixtures::v201::transaction::charging;
use serde_json::{Value, json};

fn power_monitor(kind: &str, value: f64) -> Value {
    json!({"type": kind, "value": value, "severity": 5, "component": {"name": "EVSE", "evse": {"id": 1}}, "variable": {"name": "Power"}})
}

fn power(run: &mut Run, now: u64) -> Value {
    let get = json!({"getVariableData": [{"component": {"name": "EVSE", "evse": {"id": 1}}, "variable": {"name": "Power"}}]});
    run.result(now, "GetVariables", get)["getVariableResult"][0]["attributeValue"].clone()
}

fn transaction_id(run: &Run) -> Value {
    let (_, _, started) = run
        .calls
        .iter()
        .find(|(_, action, _)| action == "TransactionEvent")
        .unwrap();
    started["transactionInfo"]["transactionId"].clone()
}

fn alerting_on_power() -> Run {
    let mut run = Run::accepted(86_400);
    monitor(&mut run, power_monitor("UpperThreshold", 1_000.0));
    charging(&mut run, 0);
    run
}

#[test]
fn given_idle_evse_when_its_power_read_then_zero() {
    let mut run = Run::accepted(86_400);

    let value = power(&mut run, 0);

    assert_eq!(value, "0");
}

#[test]
fn given_charging_ev_when_its_power_read_then_the_power_delivered() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);

    let value = power(&mut run, MINUTE);

    assert_eq!(value, "11000");
}

#[test]
fn given_booted_station_when_power_threshold_above_its_max_limit_set_then_rejected_per_n04_fr_06_and_tc_n_13()
 {
    let mut run = Run::accepted(86_400);

    let result = monitor(&mut run, power_monitor("UpperThreshold", 22_081.0));

    assert_eq!(status(&result), "Rejected");
}

#[test]
fn given_power_upper_threshold_when_energy_transfer_starts_then_alerting_with_the_transaction_per_n07_fr_06_and_tc_n_20()
 {
    let mut run = Run::accepted(86_400);
    monitor(&mut run, power_monitor("UpperThreshold", 1_000.0));

    charging(&mut run, 0);

    let events = events(&run);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["trigger"], "Alerting");
    assert_eq!(events[0]["actualValue"], "11000");
    assert_eq!(events[0]["transactionId"], transaction_id(&run));
}

#[test]
fn given_power_alert_when_charging_stops_then_cleared_with_the_transaction_per_n07_fr_02_and_tc_n_20()
 {
    let mut run = alerting_on_power();

    run.input(MINUTE, Input::Intent(Intent::Stop { connector: 1 }));
    run.answer_all(MINUTE);

    let events = events(&run);
    assert_eq!(events[1]["cleared"], true);
    assert_eq!(events[1]["actualValue"], "0");
    assert_eq!(events[1]["transactionId"], transaction_id(&run));
}

#[test]
fn given_idle_evse_with_a_power_delta_when_energy_transfer_starts_then_delta_event_per_n07_fr_18_and_tc_n_45()
 {
    let mut run = Run::accepted(86_400);
    monitor(&mut run, power_monitor("Delta", 100.0));

    charging(&mut run, 0);

    let events = events(&run);
    assert_eq!(events[0]["trigger"], "Delta");
    assert_eq!(events[0]["variableMonitoringId"], 1);
}

#[test]
fn given_power_under_its_threshold_when_the_monitor_replaced_with_a_lower_one_then_alerting_per_n07_fr_11_and_tc_n_51()
 {
    let mut run = Run::accepted(86_400);
    monitor(&mut run, power_monitor("UpperThreshold", 21_000.0));
    charging(&mut run, 0);
    let mut lower = power_monitor("UpperThreshold", 0.0);
    lower["id"] = json!(1);

    let result = monitor(&mut run, lower);
    run.answer_all(0);

    assert_eq!(status(&result), "Accepted");
    let events = events(&run);
    assert_eq!(events[0]["trigger"], "Alerting");
    assert_eq!(events[0]["actualValue"], "11000");
}

fn upper_600() -> Run {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("UpperThreshold", 600.0, 5));
    run
}

fn alerting_upper_600() -> Run {
    let mut run = upper_600();
    set_heartbeat(&mut run, "900");
    run
}

fn delta_100() -> Run {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("Delta", 100.0, 5));
    run
}

fn set(run: &mut Run, component: Value, variable: Value, value: &str) {
    let set = json!({"component": component, "variable": variable, "attributeValue": value});
    run.result(0, "SetVariables", json!({"setVariableData": [set]}));
    run.answer_all(0);
}

fn delta_on(run: &mut Run, component: Value, variable: Value) {
    let data = json!({"type": "Delta", "value": 1, "severity": 5, "component": component, "variable": variable});
    monitor(run, data);
}

#[test]
fn given_upper_threshold_when_value_reaches_it_then_no_event_per_n07_fr_16() {
    let mut run = upper_600();

    set_heartbeat(&mut run, "600");

    assert!(events(&run).is_empty(), "on the threshold is not above it");
}

#[test]
fn given_upper_threshold_when_value_exceeds_it_then_alerting_event_of_the_monitor_per_n07_fr_06_and_fr_16_and_tc_n_20()
 {
    let mut run = upper_600();

    set_heartbeat(&mut run, "900");

    let events = events(&run);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["trigger"], "Alerting");
    assert_eq!(events[0]["actualValue"], "900");
    assert_eq!(events[0]["variableMonitoringId"], 1, "N07.FR.06");
    assert_eq!(events[0]["component"]["name"], "OCPPCommCtrlr");
    assert_eq!(events[0]["eventNotificationType"], "CustomMonitor");
    assert_eq!(events[0].get("cleared"), None);
}

#[test]
fn given_raised_upper_alert_when_value_falls_back_then_cleared_event_per_n07_fr_02() {
    let mut run = alerting_upper_600();

    set_heartbeat(&mut run, "300");

    assert_eq!(events(&run).pop().unwrap()["cleared"], true);
}

#[test]
fn given_lower_threshold_when_value_reaches_it_then_no_event_per_n07_fr_17() {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("LowerThreshold", 100.0, 5));

    set_heartbeat(&mut run, "100");

    assert!(events(&run).is_empty());
}

#[test]
fn given_lower_threshold_when_value_falls_below_it_then_alerting_event_per_n07_fr_17() {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("LowerThreshold", 100.0, 5));

    set_heartbeat(&mut run, "50");

    assert_eq!(events(&run)[0]["actualValue"], "50");
}

#[test]
fn given_raised_alert_when_time_passes_without_change_then_no_new_event() {
    let mut run = alerting_upper_600();

    for second in 1..=10 {
        run.input(second * 1000, Input::Tick);
        run.answer_all(second * 1000);
    }

    assert_eq!(events(&run).len(), 1);
}

#[test]
fn given_delta_monitor_when_value_changes_by_exactly_the_delta_then_no_event_per_n07_fr_18() {
    let mut run = delta_100();

    set_heartbeat(&mut run, "400");

    assert!(events(&run).is_empty(), "exactly the delta is not more");
}

#[test]
fn given_delta_monitor_when_value_changes_by_more_than_the_delta_then_delta_event_measured_from_the_last_notice_per_n07_fr_18_and_tc_n_45()
 {
    let mut run = delta_100();
    set_heartbeat(&mut run, "400");

    set_heartbeat(&mut run, "450");
    set_heartbeat(&mut run, "500");

    let events = events(&run);
    assert_eq!(events.len(), 1, "measured from the last notice");
    assert_eq!(events[0]["trigger"], "Delta");
    assert_eq!(events[0]["actualValue"], "450");
}

#[test]
fn given_delta_on_a_boolean_variable_when_it_changes_then_delta_event_per_n07_fr_19_and_tc_n_56() {
    let mut run = Run::accepted(300);
    let (component, variable) = (
        json!({"name": "TxCtrlr"}),
        json!({"name": "StopTxOnInvalidId"}),
    );
    delta_on(&mut run, component.clone(), variable.clone());

    set(&mut run, component, variable, "false");

    assert_eq!(events(&run)[0]["actualValue"], "false");
}

#[test]
fn given_delta_on_the_password_when_it_changes_then_event_without_value_per_n07_fr_10() {
    let mut run = Run::accepted(300);
    let (component, variable) = (
        json!({"name": "SecurityCtrlr"}),
        json!({"name": "BasicAuthPassword"}),
    );
    delta_on(&mut run, component.clone(), variable.clone());

    set(&mut run, component, variable, "correct-horse-battery");

    assert_eq!(events(&run)[0]["actualValue"], "");
}

#[test]
fn given_raised_alert_when_its_monitor_replaced_with_a_higher_threshold_then_cleared_event_per_n07_fr_11()
 {
    let mut run = alerting_upper_600();
    let mut higher = data("UpperThreshold", 1000.0, 5);
    higher["id"] = json!(1);

    monitor(&mut run, higher);
    run.answer_all(0);

    assert_eq!(events(&run).pop().unwrap()["cleared"], true);
}

#[test]
fn given_cleared_monitor_with_raised_alert_when_value_falls_back_then_no_cleared_event_per_n07_fr_12()
 {
    let mut run = alerting_upper_600();
    run.result(0, "ClearVariableMonitoring", json!({"id": [1]}));

    set_heartbeat(&mut run, "300");

    assert_eq!(events(&run).len(), 1);
}

#[test]
fn given_transaction_only_monitor_without_transaction_when_value_exceeds_it_then_no_event() {
    let mut run = Run::accepted(300);
    let mut during = data("UpperThreshold", 600.0, 5);
    during["transaction"] = json!(true);
    monitor(&mut run, during);

    set_heartbeat(&mut run, "900");

    assert!(events(&run).is_empty());
}

#[test]
fn given_exceeded_transaction_only_monitor_when_a_transaction_starts_then_checked_and_alerting() {
    let mut run = Run::accepted(300);
    let mut during = data("UpperThreshold", 600.0, 5);
    during["transaction"] = json!(true);
    monitor(&mut run, during);
    set_heartbeat(&mut run, "900");
    let start = json!({"idToken": {"idToken": "APP-USER-1", "type": "Central"}, "remoteStartId": 1, "evseId": 1});

    run.result(0, "RequestStartTransaction", start);
    run.answer_all(0);

    assert_eq!(events(&run)[0]["actualValue"], "900");
}

#[test]
fn given_offline_station_with_severity_4_and_5_monitors_when_both_exceeded_and_back_online_then_only_severity_4_queued_per_n07_fr_04_and_tc_n_22()
 {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("UpperThreshold", 600.0, 4));
    monitor(&mut run, data("UpperThreshold", 700.0, 5));
    run.input(0, Input::Disconnected);
    assert!(
        run.station
            .configure("OCPPCommCtrlr.HeartbeatInterval", "900")
    );

    run.input(MINUTE, Input::Tick);
    run.input(MINUTE, Input::Connected);
    run.answer_all(MINUTE);

    let events = events(&run);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["variableMonitoringId"], 1);
}
