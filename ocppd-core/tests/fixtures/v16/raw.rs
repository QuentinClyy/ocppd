use crate::event::{Event, EventKind, Input, Output};
use crate::frame::Frame;
use crate::v16::station::{Config, Station};
use serde_json::{Value, json};

pub fn charge_point() -> Station {
    Station::new(Config {
        id: "CP1".into(),
        vendor: "ocppd".into(),
        model: "sim".into(),
        connectors: 1,
    })
}

pub fn calls(outputs: &[Output]) -> Vec<(String, String, Value)> {
    outputs
        .iter()
        .filter_map(|output| match output {
            Output::Send(Frame::Call {
                id,
                action,
                payload,
            }) => Some((id.clone(), action.clone(), payload.clone())),
            _ => None,
        })
        .collect()
}

pub fn kinds(outputs: &[Output]) -> Vec<EventKind> {
    outputs
        .iter()
        .filter_map(|output| match output {
            Output::Emit(event) => Some(v16_kind(event.clone())),
            _ => None,
        })
        .collect()
}

pub fn only_call(outputs: &[Output]) -> (String, String, Value) {
    let mut sent = calls(outputs);
    assert_eq!(sent.len(), 1, "expected one call in {outputs:?}");
    sent.remove(0)
}

pub fn result(id: &str, payload: Value) -> Input {
    Input::Frame(json!([3, id, payload]).to_string())
}

pub fn boot_response(status: &str, interval: u64) -> Value {
    json!({"status": status, "currentTime": "2026-10-19T00:00:00Z", "interval": interval})
}

pub fn v16_kind(event: Event) -> EventKind {
    event.kind
}
