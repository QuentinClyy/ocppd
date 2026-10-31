use crate::api::event::{Event, Kind};
use crate::api::io::{Input, Output};
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::event::EventKind;
use crate::station::Station;
use crate::station::connector::Connector;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

pub fn station() -> Station {
    Station {
        id: "CP1".into(),
        vendor: "ocppd".into(),
        model: "sim".into(),
        serial_number: "SN-1".into(),
        firmware_version: "0.1.0".into(),
        voltage_v: 230.0,
        max_current_a: 32.0,
        phases: 3,
        max_output_a: 500.0,
        jitter: false,
        connectors: vec![Connector::default()],
    }
}

pub fn charge_point() -> ChargePoint {
    ChargePoint::new(station()).unwrap()
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

pub fn accepted_charge_point(now: u64, interval: u64) -> ChargePoint {
    let mut charge_point = charge_point();
    let (id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let mut outputs = charge_point.handle(now, result(&id, boot_response("Accepted", interval)));
    while let Some((id, _, _)) = calls(&outputs).pop() {
        outputs = charge_point.handle(now, result(&id, json!({})));
    }
    charge_point
}

pub fn v16_kind(event: Event) -> EventKind {
    let Kind::V16(kind) = event.kind else {
        panic!("a 1.6 charge point emitted {event:?}");
    };
    kind
}

pub fn booted_and_reported(now: u64) -> (ChargePoint, Vec<Output>) {
    let mut charge_point = charge_point();
    let (boot_id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let outputs = charge_point.handle(now, result(&boot_id, boot_response("Accepted", 300)));
    (charge_point, outputs)
}

pub fn drained(charge_point: &mut ChargePoint, now: u64, mut outputs: Vec<Output>) {
    while let Some((id, _, _)) = calls(&outputs).pop() {
        outputs = charge_point.handle(now, result(&id, json!({})));
    }
}
