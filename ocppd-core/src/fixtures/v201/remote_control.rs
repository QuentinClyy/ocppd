use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub fn plug_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
}

pub fn start(run: &mut Run, evse: Option<u32>) -> Value {
    let mut request =
        json!({"idToken": {"idToken": "APP-USER-1", "type": "Central"}, "remoteStartId": 7});
    if let Some(evse) = evse {
        request["evseId"] = json!(evse);
    }
    let response = run.result(0, "RequestStartTransaction", request);
    run.answer_all(0);
    response
}

pub fn events(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "TransactionEvent")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}
