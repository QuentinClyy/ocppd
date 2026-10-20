use super::messages::boot::{
    BootNotificationRequest, BootNotificationResponse, HeartbeatRequest, RegistrationStatus,
};
use super::messages::status::{ChargePointErrorCode, ChargePointStatus, StatusNotificationRequest};
use crate::ci_string::{CiString, TooLong};
use crate::event::{Event, EventKind, Input, Output};
use crate::frame::{self, Frame};
use crate::outbox::Outbox;
use crate::station::Station;
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
    station: Station,
    boot_request: BootNotificationRequest,
    state: State,
    outbox: Outbox,
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
        Ok(ChargePoint {
            station,
            boot_request,
            state: State::Offline,
            outbox: Outbox::new(),
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
        [state_deadline, self.outbox.deadline()]
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
                self.emit(&mut out, now, EventKind::Disconnected);
            }
            Input::Tick => self.tick(&mut out, now),
            Input::Frame(text) => self.receive(&mut out, now, &text),
        }
        if let Some(frame) = self.outbox.next_frame(now) {
            out.push(Output::Send(frame));
        }
        out
    }

    fn boot(&mut self) {
        self.state = State::Booting;
        self.outbox
            .push("BootNotification", json!(self.boot_request));
    }

    fn tick(&mut self, out: &mut Vec<Output>, now: u64) {
        if let Some(action) = self.outbox.expire(now) {
            self.emit(
                out,
                now,
                EventKind::CallTimedOut {
                    action: action.clone(),
                },
            );
            self.call_failed(now, &action);
        }
        match self.state {
            State::Waiting { retry_at } if now >= retry_at => self.boot(),
            State::Accepted {
                next_heartbeat,
                interval_ms,
            } if now >= next_heartbeat => {
                self.outbox.push("Heartbeat", json!(HeartbeatRequest {}));
                self.state = State::Accepted {
                    next_heartbeat: now + interval_ms,
                    interval_ms,
                };
            }
            _ => {}
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
                Some(action) => self.call_succeeded(out, now, &action, payload),
                None => self.emit(out, now, EventKind::UnknownResponse { id }),
            },
            Frame::CallError { id, code, .. } => match self.outbox.resolve(&id) {
                Some(action) => {
                    self.emit(
                        out,
                        now,
                        EventKind::CallFailed {
                            action: action.clone(),
                            code,
                        },
                    );
                    self.call_failed(now, &action);
                }
                None => self.emit(out, now, EventKind::UnknownResponse { id }),
            },
        }
    }

    fn call_succeeded(&mut self, out: &mut Vec<Output>, now: u64, action: &str, payload: Value) {
        if action == "BootNotification" {
            self.booted(out, now, payload);
        }
    }

    fn call_failed(&mut self, now: u64, action: &str) {
        if action == "BootNotification" {
            self.state = State::Waiting {
                retry_at: now + RETRY_MS,
            };
        }
    }

    fn booted(&mut self, out: &mut Vec<Output>, now: u64, payload: Value) {
        let response: BootNotificationResponse = match serde_json::from_value(payload) {
            Ok(response) => response,
            Err(err) => {
                self.emit(
                    out,
                    now,
                    EventKind::InvalidPayload {
                        action: "BootNotification".into(),
                        reason: err.to_string(),
                    },
                );
                self.call_failed(now, "BootNotification");
                return;
            }
        };
        self.emit(
            out,
            now,
            EventKind::BootStatus {
                status: response.status,
            },
        );
        let interval_ms = match response.interval {
            0 => RETRY_MS,
            seconds => u64::from(seconds) * 1000,
        };
        match response.status {
            RegistrationStatus::Accepted => {
                self.state = State::Accepted {
                    next_heartbeat: now + interval_ms,
                    interval_ms,
                };
                for connector_id in 0..=self.station.connectors {
                    let request = StatusNotificationRequest {
                        connector_id,
                        error_code: ChargePointErrorCode::NoError,
                        info: None,
                        status: ChargePointStatus::Available,
                        timestamp: None,
                        vendor_id: None,
                        vendor_error_code: None,
                    };
                    self.outbox.push("StatusNotification", json!(request));
                }
            }
            RegistrationStatus::Pending | RegistrationStatus::Rejected => {
                self.state = State::Waiting {
                    retry_at: now + interval_ms,
                };
            }
        }
    }

    fn emit(&self, out: &mut Vec<Output>, now: u64, kind: EventKind) {
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
