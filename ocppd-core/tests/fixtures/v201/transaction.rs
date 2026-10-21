use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub fn plug_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
}

pub fn swipe(run: &mut Run, now: u64, id_tag: &str) {
    let intent = Intent::Authorize {
        connector: None,
        id_tag: id_tag.into(),
    };
    run.input(now, Input::Intent(intent));
}

pub fn accept(run: &mut Run, now: u64) {
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(now);
}

pub fn intent(run: &mut Run, now: u64, intent: Intent) {
    run.input(now, Input::Intent(intent));
    run.answer_all(now);
}

pub fn events(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "TransactionEvent")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn summaries(run: &Run) -> Vec<(String, String, Value)> {
    events(run).iter().map(summary).collect()
}

pub fn s(event_type: &str, trigger: &str, state: Value) -> (String, String, Value) {
    (event_type.into(), trigger.into(), state)
}

pub fn charging(run: &mut Run, now: u64) {
    plug_in(run, now);
    swipe(run, now, "TAG1");
    accept(run, now);
}

pub fn configure(run: &mut Run, key: &str, value: &str) {
    assert!(run.station.configure(key, value), "{key}");
}

pub fn summary(event: &Value) -> (String, String, Value) {
    (
        event["eventType"].as_str().unwrap().into(),
        event["triggerReason"].as_str().unwrap().into(),
        event["transactionInfo"]["chargingState"].clone(),
    )
}
