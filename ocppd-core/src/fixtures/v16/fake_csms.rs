use crate::api::intent::{EvSpec, Intent};
use crate::api::io::{Input, Output};
use crate::fixtures::v16::raw::v16_kind;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use crate::station::Station;
use crate::station::connector::Connector;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

pub const MINUTE: u64 = 60_000;

const AUTO_ACKED: [&str; 9] = [
    "StatusNotification",
    "SignedFirmwareStatusNotification",
    "LogStatusNotification",
    "SecurityEventNotification",
    "FirmwareStatusNotification",
    "DiagnosticsStatusNotification",
    "MeterValues",
    "Heartbeat",
    "StopTransaction",
];

pub struct Csms {
    pub charge_point: ChargePoint,
    pending: Option<(String, String, Value)>,
    pub sent: Vec<(String, Value)>,
    pub events: Vec<EventKind>,
    replies: Vec<Frame>,
    pub closed: bool,
    pub answered_by_test: Vec<&'static str>,
}

impl Csms {
    pub fn booted() -> Csms {
        Csms::booted_with(1)
    }

    pub fn booted_with(connectors: usize) -> Csms {
        let station = Station {
            id: "CP1".into(),
            vendor: "ocppd".into(),
            model: "sim".into(),
            serial_number: "CP1".into(),
            firmware_version: "0.1.0".into(),
            voltage_v: 230.0,
            max_current_a: 32.0,
            phases: 3,
            jitter: false,
            connectors: vec![Connector::default(); connectors],
        };
        let mut csms = Csms {
            charge_point: ChargePoint::new(station).unwrap(),
            pending: None,
            sent: Vec::new(),
            events: Vec::new(),
            replies: Vec::new(),
            closed: false,
            answered_by_test: Vec::new(),
        };
        csms.input(0, Input::Connected);
        csms.answer(
            0,
            json!({"status": "Accepted", "currentTime": "2026-10-20T00:00:00Z", "interval": 86400}),
        );
        csms.sent.clear();
        csms.events.clear();
        csms
    }

    pub fn disconnect(&mut self, now: u64) {
        self.pending = None;
        self.input(now, Input::Disconnected);
    }

    pub fn reconnect(&mut self, now: u64) {
        self.input(now, Input::Connected);
    }

    pub fn input(&mut self, now: u64, input: Input) {
        let outputs = self.charge_point.handle(now, input);
        for output in outputs {
            match output {
                Output::Send(Frame::Call {
                    id,
                    action,
                    payload,
                }) => {
                    assert!(
                        self.pending.is_none(),
                        "second call before the first resolved"
                    );
                    self.sent.push((action.clone(), payload.clone()));
                    self.pending = Some((id, action, payload));
                }
                Output::Send(reply) => self.replies.push(reply),
                Output::Close => self.closed = true,
                Output::Ping => {}
                Output::Emit(event) => self.events.push(v16_kind(event)),
            }
        }
        let auto_acked = matches!(&self.pending, Some((_, action, _)) if AUTO_ACKED.contains(&action.as_str())
                && !self.answered_by_test.contains(&action.as_str()));
        if auto_acked {
            self.answer(now, json!({}));
        }
    }

    pub fn tick(&mut self, now: u64) {
        self.input(now, Input::Tick);
    }

    pub fn call(&mut self, now: u64, action: &str, payload: Value) -> Frame {
        let text = json!([2, "csms-1", action, payload]).to_string();
        self.input(now, Input::Frame(text));
        let reply = self.replies.pop().expect("no reply to the CSMS call");
        assert!(self.replies.is_empty(), "more than one reply");
        reply
    }

    pub fn configure(&mut self, now: u64, key: &str, value: &str) -> Value {
        match self.call(
            now,
            "ChangeConfiguration",
            json!({"key": key, "value": value}),
        ) {
            Frame::CallResult { payload, .. } => payload["status"].clone(),
            other => panic!("expected a result, got {other:?}"),
        }
    }

    pub fn intent(&mut self, now: u64, intent: Intent) {
        self.input(now, Input::Intent(intent));
    }

    pub fn expect(&self, action: &str) -> Value {
        match &self.pending {
            Some((_, pending, payload)) if pending == action => payload.clone(),
            other => panic!("expected {action}, pending {other:?}"),
        }
    }

    pub fn fail(&mut self, now: u64) {
        let (id, _, _) = self.pending.take().expect("no pending call");
        let error = json!([4, id, "InternalError", "", {}]);
        self.input(now, Input::Frame(error.to_string()));
    }

    pub fn answer(&mut self, now: u64, payload: Value) {
        let (id, _, _) = self.pending.take().expect("no pending call");
        self.input(now, Input::Frame(json!([3, id, payload]).to_string()));
    }

    pub fn statuses(&self) -> Vec<ChargePointStatus> {
        self.events
            .iter()
            .filter_map(|kind| match kind {
                EventKind::StatusChanged { status, .. } => Some(*status),
                _ => None,
            })
            .collect()
    }

    pub fn sent(&self, action: &str) -> Vec<Value> {
        self.sent
            .iter()
            .filter(|(sent, _)| sent == action)
            .map(|(_, payload)| payload.clone())
            .collect()
    }

    pub fn charging(now: u64, ev: EvSpec) -> Csms {
        let mut csms = Csms::booted();
        csms.intent(now, Intent::PlugIn { connector: 1, ev });
        csms.intent(now, authorize("TAG1"));
        csms.answer(now, json!({"idTagInfo": {"status": "Accepted"}}));
        csms.answer(
            now,
            json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 42}),
        );
        csms
    }
}

pub fn authorize(id_tag: &str) -> Intent {
    Intent::Authorize {
        connector: Some(1),
        id_tag: id_tag.into(),
    }
}

pub fn register(meter_values: &Value) -> f64 {
    let sampled = &meter_values["meterValue"][0]["sampledValue"][0];
    assert_eq!(sampled["measurand"], "Energy.Active.Import.Register");
    sampled["value"].as_str().unwrap().parse().unwrap()
}
