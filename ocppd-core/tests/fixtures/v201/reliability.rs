use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::Value;

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

pub fn configure(run: &mut Run, key: &str, value: &str) {
    assert!(run.station.configure(key, value), "{key}");
}

pub fn status(run: &mut Run, request: Value) -> Value {
    run.result(0, "GetTransactionStatus", request)
}
