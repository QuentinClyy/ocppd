use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::{Run, boot};
use crate::v201::event::EventKind;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn get(run: &mut Run, component: Value, variable: Value) -> Value {
    let request = json!({"getVariableData": [{"component": component, "variable": variable}]});
    run.result(0, "GetVariables", request)["getVariableResult"][0].clone()
}

fn set(run: &mut Run, component: Value, variable: Value, value: &str) -> Value {
    let data = json!({"component": component, "variable": variable, "attributeValue": value});
    let request = json!({"setVariableData": [data]});
    run.result(0, "SetVariables", request)["setVariableResult"][0].clone()
}

fn named(name: &str) -> Value {
    json!({"name": name})
}

#[test]
fn given_booted_station_when_get_variables_with_mixed_items_then_each_answered_in_order_per_b06_fr_01_to_fr_11()
 {
    let mut run = Run::accepted(300);
    let request = json!({"getVariableData": [
        {"component": named("OCPPCommCtrlr"), "variable": named("HeartbeatInterval")},
        {"component": named("Nope"), "variable": named("X")},
        {"component": named("OCPPCommCtrlr"), "variable": named("Nope")},
        {"component": named("OCPPCommCtrlr"), "variable": named("HeartbeatInterval"), "attributeType": "Target"},
        {"component": named("DeviceDataCtrlr"), "variable": {"name": "ItemsPerMessage", "instance": "GetVariables"}},
    ]});

    let results = run.result(0, "GetVariables", request)["getVariableResult"].clone();

    let statuses: Vec<_> = results
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["attributeStatus"].clone())
        .collect();
    assert_eq!(
        statuses,
        vec![
            json!("Accepted"),
            json!("UnknownComponent"),
            json!("UnknownVariable"),
            json!("NotSupportedAttributeType"),
            json!("Accepted")
        ]
    );
    assert_eq!(
        results[0]["attributeValue"], "300",
        "the boot interval, B01.FR.04"
    );
    assert_eq!(results[0]["attributeType"], "Actual", "FR.11");
    assert!(
        results[1].get("attributeValue").is_none(),
        "FR.06 omits the value"
    );
    assert_eq!(results[3]["attributeType"], "Target", "FR.03");
    assert_eq!(results[4]["variable"]["instance"], "GetVariables", "FR.14");
}

#[test]
fn given_booted_station_when_variable_asked_without_its_instance_then_unknown_variable_per_b06_fr_15()
 {
    let mut run = Run::accepted(300);

    let result = get(&mut run, named("DeviceDataCtrlr"), named("ItemsPerMessage"));

    assert_eq!(result["attributeStatus"], "UnknownVariable");
}

#[test]
fn given_booted_station_when_too_many_variables_asked_then_occurrence_violation_per_b06_fr_16() {
    let mut run = Run::accepted(300);
    let item = json!({"component": named("OCPPCommCtrlr"), "variable": named("HeartbeatInterval")});
    let request = json!({"getVariableData": vec![item; 51]});

    let reply = run.csms(0, "GetVariables", request);

    match reply {
        Frame::CallError { code, .. } => assert_eq!(code, "OccurrenceConstraintViolation"),
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn given_booted_station_when_set_variables_with_mixed_values_then_each_checked_and_changes_reported_per_b05_fr_04_to_fr_10()
 {
    let mut run = Run::accepted(300);
    let comm = named("OCPPCommCtrlr");
    let cases = [
        (named("Nope"), named("X"), "1", "UnknownComponent"),
        (comm.clone(), named("Nope"), "1", "UnknownVariable"),
        (comm.clone(), named("HeartbeatInterval"), "soon", "Rejected"),
        (comm.clone(), named("HeartbeatInterval"), "-5", "Rejected"),
        (comm.clone(), named("ActiveNetworkProfile"), "1", "Rejected"),
        (named("ClockCtrlr"), named("TimeSource"), "GPS", "Rejected"),
        (
            named("ClockCtrlr"),
            named("TimeSource"),
            "RealTimeClock,Heartbeat",
            "Accepted",
        ),
        (comm.clone(), named("HeartbeatInterval"), "60", "Accepted"),
    ];

    let results: Vec<_> = cases
        .iter()
        .map(|(component, variable, value, _)| {
            set(&mut run, component.clone(), variable.clone(), value)
        })
        .collect();

    for ((component, variable, value, expected), result) in cases.iter().zip(&results) {
        assert_eq!(
            result["attributeStatus"], *expected,
            "{component} {variable} {value}"
        );
        assert_eq!(result["attributeType"], "Actual", "FR.12");
    }
    assert!(run.events.contains(&EventKind::VariableChanged {
        component: "OCPPCommCtrlr".into(),
        variable: "HeartbeatInterval".into(),
        value: "60".into(),
    }));
}

#[test]
fn given_booted_station_when_heartbeat_interval_set_then_heartbeat_rescheduled() {
    let mut run = Run::accepted(300);

    set(
        &mut run,
        named("OCPPCommCtrlr"),
        named("HeartbeatInterval"),
        "60",
    );

    assert_eq!(run.station.next_deadline(), Some(60_000));
}

#[test]
fn given_plugged_ev_when_evse_availability_state_read_then_occupied() {
    let mut run = Run::accepted(300);
    let ev = EvSpec::default();
    run.input(5_000, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    let evse = json!({"name": "EVSE", "evse": {"id": 1}});

    let result = get(&mut run, evse, named("AvailabilityState"));

    assert_eq!(result["attributeValue"], "Occupied");
}

#[test]
fn given_booted_station_when_clock_read_then_current_time() {
    let mut run = Run::accepted(300);

    let clock = get(&mut run, named("ClockCtrlr"), named("DateTime"));

    assert_eq!(clock["attributeValue"], "1970-01-01T00:00:00Z");
}

#[test]
fn given_new_station_when_configured_with_shortcut_notation_then_accepted() {
    let mut run = Run::new(1);

    let accepted = run
        .station
        .configure("OCPPCommCtrlr.OfflineThreshold", "120");

    assert!(accepted);
}

#[test]
fn given_new_station_when_read_only_variable_configured_then_refused() {
    let mut run = Run::new(1);

    let accepted = run
        .station
        .configure("OCPPCommCtrlr.ActiveNetworkProfile", "1");

    assert!(!accepted);
}

#[test]
fn given_new_station_when_unknown_key_configured_then_refused() {
    let mut run = Run::new(1);

    let accepted = run.station.configure("Nonsense", "1");

    assert!(!accepted);
}

#[test]
fn given_configured_offline_threshold_when_read_after_boot_then_configured_value() {
    let mut run = Run::new(1);
    run.station
        .configure("OCPPCommCtrlr.OfflineThreshold", "120");
    run.input(0, Input::Connected);
    run.answer(0, boot("Accepted", 300));

    let result = get(&mut run, named("OCPPCommCtrlr"), named("OfflineThreshold"));

    assert_eq!(result["attributeValue"], "120");
}
