use crate::fixtures::v201::reports::{reports, variables};
use crate::fixtures::v201::station::Run;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn get(run: &mut Run, request: Value) -> Value {
    let mut request = request;
    request["requestId"] = json!(9);
    let status = run.result(0, "GetReport", request)["status"].clone();
    run.answer_all(0);
    status
}

fn wanted(component: Value, variable: Option<Value>) -> Value {
    let mut wanted = json!({"component": component});
    if let Some(variable) = variable {
        wanted["variable"] = variable;
    }
    wanted
}

fn data(run: &Run) -> Vec<Value> {
    let reports = reports(run);
    let data = reports
        .iter()
        .flat_map(|report| report["reportData"].as_array().unwrap().clone());
    data.collect()
}

fn reservations_disabled() -> Run {
    let mut run = Run::accepted(300);
    assert!(run.station.configure("ReservationCtrlr.Enabled", "false"));
    run
}

#[test]
fn given_booted_station_when_one_variable_asked_then_reported_with_characteristics_per_b08_fr_04_fr_11_and_fr_12()
 {
    let mut run = Run::accepted(300);
    let heartbeat = wanted(
        json!({"name": "OCPPCommCtrlr"}),
        Some(json!({"name": "HeartbeatInterval"})),
    );

    let status = get(&mut run, json!({"componentVariable": [heartbeat]}));

    assert_eq!(status, "Accepted", "B08.FR.01");
    let reports = reports(&run);
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["requestId"], 9, "B08.FR.04");
    assert_eq!(reports[0]["seqNo"], 0, "B08.FR.14");
    let data = data(&run);
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["variableAttribute"][0]["value"], "300");
    assert_eq!(data[0]["variableCharacteristics"]["unit"], "s", "B08.FR.12");
}

#[test]
fn given_booted_station_when_component_without_variable_asked_then_all_its_variables_reported_per_b08_fr_03_and_fr_20()
 {
    let mut run = Run::accepted(300);
    let security = wanted(json!({"name": "SecurityCtrlr"}), None);

    get(&mut run, json!({"componentVariable": [security]}));

    let data = data(&run);
    assert!(data.len() > 5);
    assert!(
        data.iter()
            .all(|data| data["component"]["name"] == "SecurityCtrlr")
    );
    let password = data
        .iter()
        .find(|data| data["variable"]["name"] == "BasicAuthPassword")
        .unwrap();
    assert_eq!(
        password["variableAttribute"][0].get("value"),
        None,
        "WriteOnly"
    );
}

#[test]
fn given_booted_station_when_variable_without_instance_asked_then_every_instance_reported_per_b08_fr_21()
 {
    let mut run = Run::accepted(300);
    let any = wanted(
        json!({"name": "DeviceDataCtrlr"}),
        Some(json!({"name": "ItemsPerMessage"})),
    );

    get(&mut run, json!({"componentVariable": [any]}));

    assert_eq!(data(&run).len(), 3);
}

#[test]
fn given_booted_station_when_variable_with_instance_asked_then_only_that_instance_reported_per_b08_fr_21()
 {
    let mut run = Run::accepted(300);
    let one = wanted(
        json!({"name": "DeviceDataCtrlr"}),
        Some(json!({"name": "ItemsPerMessage", "instance": "GetVariables"})),
    );

    get(&mut run, json!({"componentVariable": [one]}));

    assert_eq!(data(&run).len(), 1);
}

#[test]
fn given_two_evses_when_evse_component_asked_without_evse_then_every_evse_reported_per_b08_fr_23() {
    let mut run = Run::accepted_with(2, 300);
    let every_evse = wanted(json!({"name": "EVSE"}), Some(json!({"name": "Power"})));

    get(&mut run, json!({"componentVariable": [every_evse]}));

    assert_eq!(data(&run).len(), 2);
}

#[test]
fn given_two_evses_when_connector_of_evse_2_asked_then_only_its_connector_reported_per_b08_fr_22() {
    let mut run = Run::accepted_with(2, 300);
    let evse_2 = wanted(
        json!({"name": "Connector", "evse": {"id": 2}}),
        Some(json!({"name": "ConnectorType"})),
    );

    get(&mut run, json!({"componentVariable": [evse_2]}));

    let data = data(&run);
    assert_eq!(data.len(), 1);
    assert_eq!(
        data[0]["component"]["evse"],
        json!({"id": 2, "connectorId": 1})
    );
}

#[test]
fn given_one_connector_per_evse_when_connector_2_asked_then_empty_result_set_per_b08_fr_22() {
    let mut run = Run::accepted_with(2, 300);
    let connector_2 = wanted(
        json!({"name": "Connector", "evse": {"id": 2, "connectorId": 2}}),
        None,
    );

    let status = get(&mut run, json!({"componentVariable": [connector_2]}));

    assert_eq!(status, "EmptyResultSet");
}

#[test]
fn given_booted_station_when_station_component_asked_with_evse_then_empty_result_set_per_b08_fr_22()
{
    let mut run = Run::accepted_with(2, 300);
    let station_wide = wanted(json!({"name": "OCPPCommCtrlr", "evse": {"id": 1}}), None);

    let status = get(&mut run, json!({"componentVariable": [station_wide]}));

    assert_eq!(status, "EmptyResultSet", "not an EVSE component");
}

#[test]
fn given_no_problem_when_problem_criterion_asked_then_empty_result_set_per_b08_fr_10_and_fr_15() {
    let mut run = reservations_disabled();

    let status = get(&mut run, json!({"componentCriteria": ["Problem"]}));

    assert_eq!(status, "EmptyResultSet");
}

#[test]
fn given_reservations_disabled_when_enabled_criterion_asked_then_enabled_and_flagless_components_reported_per_b08_fr_09()
 {
    let mut run = reservations_disabled();

    get(&mut run, json!({"componentCriteria": ["Enabled"]}));

    let names = variables(&reports(&run));
    assert!(
        names.contains(&"SecurityCtrlr.SecurityProfile".to_owned()),
        "no Enabled variable"
    );
    assert!(
        names.contains(&"SmartChargingCtrlr.Entries".to_owned()),
        "enabled"
    );
    assert!(
        !names
            .iter()
            .any(|name| name.starts_with("ReservationCtrlr")),
        "disabled"
    );
}

#[test]
fn given_reservations_disabled_when_several_criteria_asked_then_any_criterion_selects_per_b08_fr_13()
 {
    let mut run = reservations_disabled();

    get(
        &mut run,
        json!({"componentCriteria": ["Problem", "Enabled", "Available"]}),
    );

    let names = variables(&reports(&run));
    assert!(
        names
            .iter()
            .any(|name| name.starts_with("ReservationCtrlr"))
    );
}

#[test]
fn given_booted_station_when_active_criterion_asked_then_components_without_flag_count_as_active_per_b08_fr_07()
 {
    let mut run = Run::accepted(300);

    get(&mut run, json!({"componentCriteria": ["Active"]}));

    let names = variables(&reports(&run));
    assert!(names.contains(&"ReservationCtrlr.Enabled".to_owned()));
    assert!(names.contains(&"OCPPCommCtrlr.HeartbeatInterval".to_owned()));
}

#[test]
fn given_reservations_disabled_when_criteria_and_its_component_asked_then_empty_result_set_per_b08_fr_05()
 {
    let mut run = reservations_disabled();
    let request = json!({
        "componentCriteria": ["Enabled"],
        "componentVariable": [wanted(json!({"name": "ReservationCtrlr"}), None)],
    });

    let status = get(&mut run, request);

    assert_eq!(status, "EmptyResultSet");
}

#[test]
fn given_booted_station_when_too_many_component_variables_asked_then_occurrence_violation_per_b08_fr_17()
 {
    let mut run = Run::accepted(300);
    let one = wanted(json!({"name": "OCPPCommCtrlr"}), None);
    let request = json!({"requestId": 9, "componentVariable": vec![one; 101]});

    let reply = run.csms(0, "GetReport", request);

    match reply {
        Frame::CallError { code, .. } => assert_eq!(code, "OccurrenceConstraintViolation"),
        other => panic!("expected an error, got {other:?}"),
    }
}
