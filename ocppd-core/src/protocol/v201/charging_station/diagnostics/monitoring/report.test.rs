use crate::fixtures::v201::monitors::{data, monitor};
use crate::fixtures::v201::station::Run;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn installed() -> Run {
    let mut run = Run::accepted(300);
    monitor(&mut run, data("UpperThreshold", 600.0, 5));
    monitor(&mut run, data("Delta", 100.0, 5));
    let mut periodic = data("Periodic", 60.0, 5);
    periodic["variable"] = json!({"name": "OfflineThreshold"});
    monitor(&mut run, periodic);
    run
}

fn report(run: &mut Run, request: Value) -> Value {
    run.calls.clear();
    let mut request = request;
    request["requestId"] = json!(3);
    let status = run.result(0, "GetMonitoringReport", request)["status"].clone();
    run.answer_all(0);
    status
}

fn reported(run: &Run) -> Vec<(String, Vec<i64>)> {
    let reports = run
        .calls
        .iter()
        .filter(|(_, action, _)| action == "NotifyMonitoringReport");
    reports
        .flat_map(|(_, _, report)| {
            assert_eq!(report["requestId"], 3, "N02.FR.04");
            report["monitor"].as_array().unwrap().clone()
        })
        .map(|data| {
            let monitors = data["variableMonitoring"].as_array().unwrap().iter();
            let ids = monitors.map(|monitor| monitor["id"].as_i64().unwrap());
            (
                data["variable"]["name"].as_str().unwrap().to_owned(),
                ids.collect(),
            )
        })
        .collect()
}

fn names(pairs: &[(&str, &[i64])]) -> Vec<(String, Vec<i64>)> {
    let pairs = pairs.iter();
    pairs
        .map(|(name, ids)| (name.to_string(), ids.to_vec()))
        .collect()
}

fn set_base(run: &mut Run, base: &str) -> Value {
    let request = json!({"monitoringBase": base});
    run.result(0, "SetMonitoringBase", request)["status"].clone()
}

#[test]
fn given_three_monitors_when_all_reported_then_every_monitor_per_variable_in_one_part_per_n02_fr_01_fr_03_fr_09_fr_11_and_tc_n_47()
 {
    let mut run = installed();

    let status = report(&mut run, json!({}));

    assert_eq!(status, "Accepted", "N02.FR.01");
    let (_, _, first) = run
        .calls
        .iter()
        .find(|(_, action, _)| action == "NotifyMonitoringReport")
        .unwrap();
    assert_eq!(first["seqNo"], 0, "N02.FR.09");
    assert_eq!(first["tbc"], false);
    assert_eq!(
        first["monitor"][0]["variableMonitoring"][0],
        json!({"id": 1, "transaction": false, "value": 600.0, "type": "UpperThreshold", "severity": 5})
    );
    assert_eq!(
        reported(&run),
        names(&[("HeartbeatInterval", &[1, 2]), ("OfflineThreshold", &[3])])
    );
}

#[test]
fn given_three_monitors_when_reported_by_type_then_each_criterion_selects_its_type_per_n02_fr_06_and_fr_12_to_fr_14_and_tc_n_01()
 {
    let mut run = installed();
    let cases = [
        ("ThresholdMonitoring", "HeartbeatInterval", 1),
        ("DeltaMonitoring", "HeartbeatInterval", 2),
        ("PeriodicMonitoring", "OfflineThreshold", 3),
    ];

    let reports: Vec<_> = cases
        .iter()
        .map(|(criterion, _, _)| {
            report(&mut run, json!({"monitoringCriteria": [criterion]}));
            reported(&run)
        })
        .collect();

    for ((criterion, variable, id), reported) in cases.iter().zip(&reports) {
        assert_eq!(*reported, names(&[(variable, &[*id])]), "{criterion}");
    }
}

#[test]
fn given_three_monitors_when_reported_for_a_component_then_its_variables_per_n02_fr_16_and_tc_n_03()
{
    let mut run = installed();
    let component = json!({"componentVariable": [{"component": {"name": "OCPPCommCtrlr"}}]});

    report(&mut run, component);

    assert_eq!(reported(&run).len(), 2);
}

#[test]
fn given_three_monitors_when_reported_for_one_variable_then_only_its_monitors_per_n02_fr_08_and_tc_n_02()
 {
    let mut run = installed();
    let variable = json!({"componentVariable": [{"component": {"name": "OCPPCommCtrlr"}, "variable": {"name": "OfflineThreshold"}}]});

    report(&mut run, variable);

    assert_eq!(reported(&run), names(&[("OfflineThreshold", &[3])]));
}

#[test]
fn given_three_monitors_when_criteria_and_variable_select_nothing_then_empty_result_set_per_n02_fr_05_and_fr_10()
 {
    let mut run = installed();
    let both = json!({
        "monitoringCriteria": ["ThresholdMonitoring"],
        "componentVariable": [{"component": {"name": "OCPPCommCtrlr"}, "variable": {"name": "OfflineThreshold"}}],
    });

    let status = report(&mut run, both);

    assert_eq!(status, "EmptyResultSet");
    assert!(reported(&run).is_empty());
}

#[test]
fn given_no_monitors_when_all_reported_then_empty_result_set_per_n02_fr_05() {
    let mut run = Run::accepted(300);

    let status = report(&mut run, json!({}));

    assert_eq!(status, "EmptyResultSet");
}

#[test]
fn given_booted_station_when_report_asked_for_101_component_variables_then_occurrence_violation_per_n02_fr_07()
 {
    let mut run = Run::accepted(300);
    let one = json!({"component": {"name": "OCPPCommCtrlr"}});
    let request = json!({"requestId": 3, "componentVariable": vec![one; 101]});

    let reply = run.csms(0, "GetMonitoringReport", request);

    match reply {
        Frame::CallError { code, .. } => assert_eq!(code, "OccurrenceConstraintViolation"),
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn given_three_monitors_when_each_monitoring_base_set_then_accepted_and_active_per_n03_fr_01() {
    for base in ["All", "FactoryDefault", "HardWiredOnly"] {
        let mut run = installed();

        let status = set_base(&mut run, base);

        assert_eq!(status, "Accepted", "{base}");
        let get = json!({"getVariableData": [{"component": {"name": "MonitoringCtrlr"}, "variable": {"name": "ActiveMonitoringBase"}}]});
        let response = run.result(0, "GetVariables", get);
        assert_eq!(response["getVariableResult"][0]["attributeValue"], base);
    }
}

#[test]
fn given_monitoring_base_set_when_monitors_reported_then_custom_monitors_kept_only_for_all_per_n03_fr_03_to_fr_05_and_tc_n_41()
 {
    for (base, left) in [("All", 2), ("FactoryDefault", 0), ("HardWiredOnly", 0)] {
        let mut run = installed();
        set_base(&mut run, base);

        report(&mut run, json!({}));

        assert_eq!(reported(&run).len(), left, "{base}");
    }
}
