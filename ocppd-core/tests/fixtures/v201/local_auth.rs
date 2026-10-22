use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub const CARD: &str = "04A2B3C4";

pub fn swipe(run: &mut Run, now: u64, id_tag: &str) {
    let intent = Intent::Authorize {
        connector: Some(1),
        id_tag: id_tag.into(),
    };
    run.input(now, Input::Intent(intent));
}

pub fn session(run: &mut Run, now: u64, status: &str) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
    swipe(run, now, CARD);
    if run.last_call().1 == "Authorize" {
        run.answer(now, json!({"idTokenInfo": {"status": status}}));
    }
    run.answer_all(now);
    run.input(now + 1, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(now + 1);
}

pub fn authorizes(run: &Run) -> usize {
    run.actions()
        .iter()
        .filter(|action| *action == "Authorize")
        .count()
}

pub fn events(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "TransactionEvent")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn list(entries: Value, version: i32, update: &str) -> Value {
    let mut request = json!({"versionNumber": version, "updateType": update});
    if !entries.is_null() {
        request["localAuthorizationList"] = entries;
    }
    request
}

pub fn entry(id: &str, status: &str) -> Value {
    json!({"idToken": {"idToken": id, "type": "ISO14443"}, "idTokenInfo": {"status": status}})
}

pub fn send(run: &mut Run, request: Value) -> Value {
    run.result(0, "SendLocalList", request)["status"].clone()
}

pub fn version(run: &mut Run) -> Value {
    run.result(0, "GetLocalListVersion", json!({}))["versionNumber"].clone()
}

pub fn configure(run: &mut Run, key: &str, value: &str) {
    assert!(run.station.configure(key, value), "{key}");
}

pub fn plug_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
}

pub fn unplug(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(now);
}

pub fn session_answered(run: &mut Run, id_token_info: Value, transaction_info: Option<Value>) {
    plug_in(run, 0);
    swipe(run, 0, CARD);
    run.answer(0, json!({"idTokenInfo": id_token_info}));
    if let Some(info) = transaction_info {
        run.answer(0, json!({"idTokenInfo": info}));
    }
    run.answer_all(0);
    unplug(run, 1);
}

pub fn list_v3() -> Run {
    let mut run = Run::accepted(86_400);
    let full = list(
        json!([entry(CARD, "Accepted"), entry("04B5C6D7", "Accepted")]),
        3,
        "Full",
    );
    send(&mut run, full);
    run
}

pub fn clear_cache(run: &mut Run) -> Value {
    run.result(0, "ClearCache", json!({}))["status"].clone()
}
