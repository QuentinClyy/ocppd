use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

pub fn charging(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
    let intent = Intent::Authorize {
        connector: None,
        id_tag: "04A2B3C4".into(),
    };
    run.input(now, Input::Intent(intent));
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(now);
}

pub fn sent(run: &Run, action: &str) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, sent, _)| sent == action)
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

pub fn configure(run: &mut Run, settings: &[(&str, &str)]) {
    for (key, value) in settings {
        assert!(run.station.configure(key, value), "{key}");
    }
}

pub fn unplug(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(now);
}
