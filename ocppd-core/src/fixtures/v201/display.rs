use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub fn message(id: i32, priority: &str, content: &str) -> Value {
    json!({"id": id, "priority": priority, "message": {"format": "UTF8", "content": content}})
}

pub fn set(run: &mut Run, message: Value) -> Value {
    let status = run.result(0, "SetDisplayMessage", json!({"message": message}))["status"].clone();
    run.answer_all(0);
    status
}

pub fn get(run: &mut Run, request: Value) -> (Value, Vec<Value>) {
    run.calls.clear();
    let mut request = request;
    request["requestId"] = json!(4);
    let status = run.result(0, "GetDisplayMessages", request)["status"].clone();
    run.answer_all(0);
    let reports = run
        .calls
        .iter()
        .filter(|(_, action, _)| action == "NotifyDisplayMessages");
    let messages = reports.flat_map(|(_, _, report)| {
        assert_eq!(report["requestId"], 4, "O03.FR.05");
        assert_eq!(report["tbc"], false);
        report["messageInfo"].as_array().unwrap().clone()
    });
    (status, messages.collect())
}

pub fn ids(messages: &[Value]) -> Vec<i64> {
    messages
        .iter()
        .map(|message| message["id"].as_i64().unwrap())
        .collect()
}

pub fn count(run: &mut Run) -> Value {
    let get = json!({"getVariableData": [{"component": {"name": "DisplayMessageCtrlr"}, "variable": {"name": "DisplayMessages"}}]});
    run.result(0, "GetVariables", get)["getVariableResult"][0]["attributeValue"].clone()
}
