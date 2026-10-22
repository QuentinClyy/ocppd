use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub fn heartbeat() -> (Value, Value) {
    (
        json!({"name": "OCPPCommCtrlr"}),
        json!({"name": "HeartbeatInterval"}),
    )
}

pub fn data(kind: &str, value: f64, severity: u32) -> Value {
    let (component, variable) = heartbeat();
    json!({"type": kind, "value": value, "severity": severity, "component": component, "variable": variable})
}

pub fn monitor(run: &mut Run, data: Value) -> Value {
    let request = json!({"setMonitoringData": [data]});
    run.result(0, "SetVariableMonitoring", request)["setMonitoringResult"][0].clone()
}

pub fn set_heartbeat(run: &mut Run, value: &str) {
    let (component, variable) = heartbeat();
    let data = json!({"component": component, "variable": variable, "attributeValue": value});
    run.result(0, "SetVariables", json!({"setVariableData": [data]}));
    run.answer_all(0);
}

pub fn events(run: &Run) -> Vec<Value> {
    let notifications = run
        .calls
        .iter()
        .filter(|(_, action, _)| action == "NotifyEvent");
    notifications
        .flat_map(|(_, _, payload)| {
            assert_eq!(payload["seqNo"], 0, "N07.FR.07");
            payload["eventData"].as_array().unwrap().clone()
        })
        .collect()
}

pub fn status(result: &Value) -> &str {
    result["status"].as_str().unwrap()
}
