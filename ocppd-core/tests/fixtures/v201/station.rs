use crate::api::event::Kind;
use crate::api::io::{Input, Output};
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::station::Station;
use crate::station::connector::Connector;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

pub const MINUTE: u64 = 60_000;
const MAX_ANSWERS: usize = 1000;

pub struct Run {
    pub station: ChargingStation,
    pub calls: Vec<(String, String, Value)>,
    pending: Option<String>,
    pub events: Vec<EventKind>,
    pub replies: Vec<Frame>,
    pub closed: bool,
    pub pings: usize,
}

impl Run {
    pub fn new(connectors: usize) -> Run {
        let station = Station {
            id: "CS1".into(),
            vendor: "ocppd".into(),
            model: "sim".into(),
            serial_number: "SN-1".into(),
            firmware_version: "0.1.0".into(),
            voltage_v: 230.0,
            max_current_a: 32.0,
            phases: 3,
            jitter: false,
            connectors: vec![Connector::default(); connectors],
        };
        Run {
            station: ChargingStation::new(station).unwrap(),
            calls: Vec::new(),
            pending: None,
            events: Vec::new(),
            replies: Vec::new(),
            closed: false,
            pings: 0,
        }
    }

    pub fn accepted(interval: u32) -> Run {
        Run::accepted_with(1, interval)
    }

    pub fn accepted_with(connectors: usize, interval: u32) -> Run {
        let mut run = Run::new(connectors);
        run.input(0, Input::Connected);
        run.answer(0, boot("Accepted", interval));
        run.answer_all(0);
        run.calls.clear();
        run.events.clear();
        run
    }

    pub fn input(&mut self, now: u64, input: Input) {
        for output in self.station.handle(now, input) {
            match output {
                Output::Send(Frame::Call {
                    id,
                    action,
                    payload,
                }) => {
                    self.pending = Some(id.clone());
                    self.calls.push((id, action, payload));
                }
                Output::Send(reply) => self.replies.push(reply),
                Output::Emit(event) => match event.kind {
                    Kind::V201(kind) => self.events.push(kind),
                    other => panic!("a 2.0.1 station emitted {other:?}"),
                },
                Output::Close => self.closed = true,
                Output::Ping => self.pings += 1,
            }
        }
    }

    pub fn csms(&mut self, now: u64, action: &str, payload: Value) -> Frame {
        let call = json!([2, "csms-1", action, payload]).to_string();
        self.input(now, Input::Frame(call));
        self.replies.pop().expect("no reply to the CSMS call")
    }

    pub fn result(&mut self, now: u64, action: &str, payload: Value) -> Value {
        match self.csms(now, action, payload) {
            Frame::CallResult { payload, .. } => payload,
            other => panic!("{action}: expected a result, got {other:?}"),
        }
    }

    pub fn reconnect(&mut self, now: u64) {
        self.input(now, Input::Disconnected);
        self.input(now, Input::Connected);
    }

    pub fn boot_request(&self) -> Value {
        let (_, action, payload) = self.last_call();
        assert_eq!(action, "BootNotification");
        payload
    }

    pub fn last_call(&self) -> (String, String, Value) {
        self.calls.last().expect("no call sent").clone()
    }

    pub fn actions(&self) -> Vec<String> {
        self.calls
            .iter()
            .map(|(_, action, _)| action.clone())
            .collect()
    }

    pub fn answer(&mut self, now: u64, payload: Value) {
        let id = self.pending.take().expect("no pending call");
        self.input(now, Input::Frame(json!([3, id, payload]).to_string()));
    }

    pub fn fail(&mut self, now: u64) {
        let id = self.pending.take().expect("no pending call");
        let error = json!([4, id, "InternalError", "", {}]);
        self.input(now, Input::Frame(error.to_string()));
    }

    pub fn answer_all(&mut self, now: u64) {
        for _ in 0..MAX_ANSWERS {
            if self.pending.is_none() {
                return;
            }
            self.answer(now, json!({}));
        }
        panic!("still calling after {MAX_ANSWERS} answers");
    }
}

pub fn boot(status: &str, interval: u32) -> Value {
    json!({"status": status, "currentTime": "2026-10-21T00:00:00Z", "interval": interval})
}
