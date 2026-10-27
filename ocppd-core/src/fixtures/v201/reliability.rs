use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub fn plug_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
}

pub fn swipe(run: &mut Run, now: u64, id_tag: &str) {
    let intent = Intent::Authorize {
        connector: None,
        id_tag: id_tag.into(),
    };
    run.input(now, Input::Intent(intent));
}

pub fn events(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "TransactionEvent")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn last(run: &Run) -> (String, String, Value, Value) {
    let event = events(run).pop().expect("no TransactionEvent");
    (
        event["eventType"].as_str().unwrap().into(),
        event["triggerReason"].as_str().unwrap().into(),
        event["transactionInfo"]["chargingState"].clone(),
        event["transactionInfo"]["stoppedReason"].clone(),
    )
}

pub fn configure(run: &mut Run, key: &str, value: &str) {
    assert!(run.station.configure(key, value), "{key}");
}

pub fn status(run: &mut Run, request: Value) -> Value {
    run.result(0, "GetTransactionStatus", request)
}

pub fn charging(run: &mut Run, now: u64) {
    plug_in(run, now);
    run.answer_all(now);
    swipe(run, now, "04A2B3C4");
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "FLEET", "type": "Central"}}}));
    run.answer_all(now);
}

pub fn expected(
    event_type: &str,
    trigger: &str,
    state: Value,
    reason: Value,
) -> (String, String, Value, Value) {
    (event_type.into(), trigger.into(), state, reason)
}
