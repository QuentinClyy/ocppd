use crate::fixtures::v201::reports::{reports, variables};
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn get_base_report(run: &mut Run, base: &str) -> Value {
    let request = json!({"requestId": 7, "reportBase": base});
    let status = run.result(0, "GetBaseReport", request)["status"].clone();
    run.answer_all(0);
    status
}

#[test]
fn given_booted_station_when_full_inventory_asked_then_every_variable_reported_with_characteristics_per_b07_fr_08()
 {
    let mut run = Run::accepted(300);

    let status = get_base_report(&mut run, "FullInventory");

    let reports = reports(&run);
    assert_eq!(status, "Accepted", "FR.01");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["requestId"], 7, "FR.04");
    assert_eq!(reports[0]["seqNo"], 0, "FR.10");
    assert_eq!(reports[0]["tbc"], false);
    let all = variables(&reports);
    for required in [
        "OCPPCommCtrlr.HeartbeatInterval",
        "DeviceDataCtrlr.ItemsPerMessage",
        "ClockCtrlr.DateTime",
        "SecurityCtrlr.SecurityProfile",
        "EVSE.Power",
        "Connector.ConnectorType",
    ] {
        assert!(all.contains(&required.to_owned()), "{required}");
    }
    let first = &reports[0]["reportData"][0];
    assert!(first["variableCharacteristics"]["dataType"].is_string());
    assert_eq!(first["variableAttribute"][0]["type"], "Actual", "FR.11");
}

#[test]
fn given_booted_station_when_configuration_inventory_asked_then_only_writable_variables_reported_per_b07_fr_07()
 {
    let mut run = Run::accepted(300);

    get_base_report(&mut run, "ConfigurationInventory");

    for data in reports(&run)[0]["reportData"].as_array().unwrap() {
        assert_ne!(
            data["variableAttribute"][0]["mutability"], "ReadOnly",
            "{data}"
        );
        assert_eq!(data["variableAttribute"][0]["persistent"], true);
    }
}

#[test]
fn given_two_evses_when_summary_inventory_asked_then_availability_reported_per_b07_fr_09() {
    let mut run = Run::accepted_with(2, 300);

    get_base_report(&mut run, "SummaryInventory");

    assert_eq!(
        variables(&reports(&run)),
        vec![
            "ChargingStation.AvailabilityState",
            "EVSE.AvailabilityState",
            "Connector.AvailabilityState",
            "EVSE.AvailabilityState",
            "Connector.AvailabilityState"
        ]
    );
}

#[test]
fn given_twelve_evses_when_full_inventory_asked_then_split_with_seq_no_and_tbc_per_b07_fr_10() {
    let mut run = Run::accepted_with(12, 300);

    get_base_report(&mut run, "FullInventory");

    let reports = reports(&run);
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0]["reportData"].as_array().unwrap().len(), 100);
    assert_eq!(
        (reports[0]["seqNo"].clone(), reports[0]["tbc"].clone()),
        (json!(0), json!(true))
    );
    assert_eq!(
        (reports[1]["seqNo"].clone(), reports[1]["tbc"].clone()),
        (json!(1), json!(false))
    );
}
