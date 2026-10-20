use super::messages::boot::{
    BootNotificationRequest, BootNotificationResponse, HeartbeatRequest, RegistrationStatus,
};
use super::messages::status::{ChargePointErrorCode, ChargePointStatus, StatusNotificationRequest};
use super::transaction::Transaction;
use crate::ci_string::{CiString, TooLong};
use crate::event::{Event, EventKind, Input, Output};
use crate::frame::{self, Frame};
use crate::outbox::{Call, Outbox};
use crate::station::Station;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

const RETRY_MS: u64 = 60_000;

enum State {
    Offline,
    Booting,
    Waiting {
        retry_at: u64,
    },
    Accepted {
        next_heartbeat: u64,
        interval_ms: u64,
    },
}

pub struct ChargePoint {
    pub(super) station: Station,
    pub(super) outbox: Outbox,
    pub(super) transactions: Vec<Transaction>,
    pub(super) reported: Vec<Option<ChargePointStatus>>,
    boot_request: BootNotificationRequest,
    state: State,
}

impl ChargePoint {
    pub fn new(station: Station) -> Result<Self, TooLong> {
        let boot_request = BootNotificationRequest {
            charge_point_vendor: CiString::new(&station.vendor)?,
            charge_point_model: CiString::new(&station.model)?,
            charge_point_serial_number: Some(CiString::new(&station.serial_number)?),
            charge_box_serial_number: None,
            firmware_version: Some(CiString::new(&station.firmware_version)?),
            iccid: None,
            imsi: None,
            meter_type: None,
            meter_serial_number: None,
        };
        let connectors = station.connectors.len();
        Ok(ChargePoint {
            station,
            outbox: Outbox::new(),
            transactions: vec![Transaction::Idle; connectors],
            reported: vec![None; connectors],
            boot_request,
            state: State::Offline,
        })
    }

    pub fn station(&self) -> &Station {
        &self.station
    }

    pub fn next_deadline(&self) -> Option<u64> {
        let state_deadline = match self.state {
            State::Waiting { retry_at } => Some(retry_at),
            State::Accepted { next_heartbeat, .. } => Some(next_heartbeat),
            State::Offline | State::Booting => None,
        };
        [
            state_deadline,
            self.outbox.deadline(),
            self.transaction_deadline(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub fn handle(&mut self, now: u64, input: Input) -> Vec<Output> {
        let mut out = Vec::new();
        match input {
            Input::Connected => {
                self.emit(&mut out, now, EventKind::Connected);
                self.boot();
            }
            Input::Disconnected => {
                self.state = State::Offline;
                self.outbox.clear();
                self.connection_lost();
                self.emit(&mut out, now, EventKind::Disconnected);
            }
            Input::Tick => self.tick(&mut out, now),
            Input::Frame(text) => self.receive(&mut out, now, &text),
            Input::Intent(intent) if self.is_accepted() => self.intent(&mut out, now, intent),
            Input::Intent(intent) => {
                self.reject(&mut out, now, intent.connector(), "not registered")
            }
        }
        if self.is_accepted() {
            self.report_statuses(&mut out, now);
        }
        if let Some(frame) = self.outbox.next_frame(now) {
            out.push(Output::Send(frame));
        }
        out
    }

    fn is_accepted(&self) -> bool {
        matches!(self.state, State::Accepted { .. })
    }

    fn boot(&mut self) {
        self.state = State::Booting;
        self.outbox
            .push("BootNotification", 0, json!(self.boot_request));
    }

    fn tick(&mut self, out: &mut Vec<Output>, now: u64) {
        if let Some(call) = self.outbox.expire(now) {
            let action = call.action.clone();
            self.emit(out, now, EventKind::CallTimedOut { action });
            self.call_failed(now, &call);
        }
        match self.state {
            State::Waiting { retry_at } if now >= retry_at => self.boot(),
            State::Accepted {
                next_heartbeat,
                interval_ms,
            } if now >= next_heartbeat => {
                self.outbox.push("Heartbeat", 0, json!(HeartbeatRequest {}));
                self.state = State::Accepted {
                    next_heartbeat: now + interval_ms,
                    interval_ms,
                };
            }
            _ => {}
        }
        if self.is_accepted() {
            self.sample_meters(out, now);
        }
    }

    fn receive(&mut self, out: &mut Vec<Output>, now: u64, text: &str) {
        let frame = match frame::parse(text) {
            Ok(frame) => frame,
            Err(err) => {
                let reason = err.to_string();
                self.emit(out, now, EventKind::FrameRejected { reason });
                return;
            }
        };
        match frame {
            Frame::Call { id, action, .. } => out.push(Output::Send(Frame::CallError {
                id,
                code: "NotImplemented".into(),
                description: format!("{action} is not implemented"),
                details: json!({}),
            })),
            Frame::CallResult { id, payload } => match self.outbox.resolve(&id) {
                Some(call) => self.call_succeeded(out, now, &call, payload),
                None => self.emit(out, now, EventKind::UnknownResponse { id }),
            },
            Frame::CallError { id, code, .. } => match self.outbox.resolve(&id) {
                Some(call) => {
                    let action = call.action.clone();
                    self.emit(out, now, EventKind::CallFailed { action, code });
                    self.call_failed(now, &call);
                }
                None => self.emit(out, now, EventKind::UnknownResponse { id }),
            },
        }
    }

    fn call_succeeded(&mut self, out: &mut Vec<Output>, now: u64, call: &Call, payload: Value) {
        match call.action.as_str() {
            "BootNotification" => self.booted(out, now, payload),
            "Authorize" | "StartTransaction" | "StopTransaction" => {
                self.transaction_reply(out, now, call, payload)
            }
            _ => {}
        }
    }

    fn call_failed(&mut self, now: u64, call: &Call) {
        match call.action.as_str() {
            "BootNotification" => {
                self.state = State::Waiting {
                    retry_at: now + RETRY_MS,
                }
            }
            "Authorize" | "StartTransaction" | "StopTransaction" => self.transaction_failed(call),
            _ => {}
        }
    }

    fn booted(&mut self, out: &mut Vec<Output>, now: u64, payload: Value) {
        let Some(response) =
            self.decode::<BootNotificationResponse>(out, now, "BootNotification", payload)
        else {
            self.state = State::Waiting {
                retry_at: now + RETRY_MS,
            };
            return;
        };
        let status = response.status;
        self.emit(out, now, EventKind::BootStatus { status });
        let interval_ms = match response.interval {
            0 => RETRY_MS,
            seconds => u64::from(seconds) * 1000,
        };
        self.state = match status {
            RegistrationStatus::Accepted => State::Accepted {
                next_heartbeat: now + interval_ms,
                interval_ms,
            },
            RegistrationStatus::Pending | RegistrationStatus::Rejected => State::Waiting {
                retry_at: now + interval_ms,
            },
        };
        if status == RegistrationStatus::Accepted {
            self.status_notification(0, ChargePointStatus::Available);
            self.reported.fill(None);
        }
    }

    pub(super) fn status_notification(&mut self, connector_id: u32, status: ChargePointStatus) {
        let request = StatusNotificationRequest {
            connector_id,
            error_code: ChargePointErrorCode::NoError,
            info: None,
            status,
            timestamp: None,
            vendor_id: None,
            vendor_error_code: None,
        };
        self.outbox
            .push("StatusNotification", connector_id, json!(request));
    }

    pub(super) fn decode<T: DeserializeOwned>(
        &self,
        out: &mut Vec<Output>,
        now: u64,
        action: &str,
        payload: Value,
    ) -> Option<T> {
        match serde_json::from_value(payload) {
            Ok(response) => Some(response),
            Err(err) => {
                let action = action.to_owned();
                let reason = err.to_string();
                self.emit(out, now, EventKind::InvalidPayload { action, reason });
                None
            }
        }
    }

    pub(super) fn reject(&self, out: &mut Vec<Output>, now: u64, connector: u32, reason: &str) {
        let reason = reason.to_owned();
        self.emit(out, now, EventKind::IntentRejected { connector, reason });
    }

    pub(super) fn emit(&self, out: &mut Vec<Output>, now: u64, kind: EventKind) {
        out.push(Output::Emit(Event {
            station: self.station.id.clone(),
            at: now,
            kind,
        }));
    }
}

#[cfg(test)]
#[path = "charge_point.test.rs"]
mod tests;
