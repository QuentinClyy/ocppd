mod configuration;
mod csms_call;
mod driver;
mod held;
mod meter_values;
mod registration;
mod remote_control;
mod reporting;
mod retry;
mod transaction;

use crate::api::event::{Event, EventKind};
use crate::api::intent::Intent;
use crate::api::io::{Input, Output};
use crate::station::Station;
use crate::v16::messages::core::boot::BootNotificationRequest;
use crate::v16::messages::core::remote_control::ResetType;
use crate::v16::messages::core::status::{
    ChargePointErrorCode, ChargePointStatus, StatusNotificationRequest,
};
use crate::wire::ci_string::{CiString, TooLong};
use crate::wire::frame::{self, Frame};
use crate::wire::outbox::{Call, Outbox, Sent};
use configuration::Configuration;
use held::Held;
use registration::Registration;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use transaction::Transaction;

pub struct ChargePoint {
    station: Station,
    outbox: Outbox,
    transactions: Vec<Transaction>,
    reported: Vec<Option<ChargePointStatus>>,
    resend_statuses: bool,
    inoperative: Vec<bool>,
    resetting: Option<ResetType>,
    held: Option<Held>,
    configuration: Configuration,
    boot_request: BootNotificationRequest,
    registration: Registration,
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
            reported: vec![None; connectors + 1],
            resend_statuses: false,
            inoperative: vec![false; connectors + 1],
            resetting: None,
            held: None,
            configuration: Configuration::new(connectors)?,
            boot_request,
            registration: Registration::Offline,
        })
    }

    pub fn station(&self) -> &Station {
        &self.station
    }

    pub fn next_deadline(&self) -> Option<u64> {
        [
            self.registration_deadline(),
            self.outbox.deadline(self.is_accepted()),
            self.transaction_deadline(),
            self.held_deadline(),
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
                self.connected(now);
            }
            Input::Disconnected => {
                self.disconnected();
                self.outbox.keep_transaction_messages();
                self.connection_lost();
                self.emit(&mut out, now, EventKind::Disconnected);
            }
            Input::Tick => self.tick(&mut out, now),
            Input::Frame(text) => self.receive(&mut out, now, &text),
            Input::Intent(Intent::Authorize { connector, .. }) if !self.is_accepted() => {
                self.reject(&mut out, now, connector, "not registered")
            }
            Input::Intent(intent) => self.intent(&mut out, now, intent),
        }
        if self.is_accepted() {
            self.report_statuses(&mut out, now);
        }
        if self.reset_due(now) {
            self.rebooted();
            out.push(Output::Close);
        }
        if self.is_connected() {
            if let Some(frame) = self.outbox.next_frame(now, self.is_accepted()) {
                out.push(Output::Send(frame));
            }
        }
        out
    }

    fn tick(&mut self, out: &mut Vec<Output>, now: u64) {
        if let Some(sent) = self.outbox.expire(now) {
            let action = sent.call.action.clone();
            self.emit(out, now, EventKind::CallTimedOut { action });
            self.call_failed(out, now, sent);
        }
        self.registration_tick(now);
        self.sample_meters(out, now);
        self.expire_held(out, now);
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
            Frame::Call {
                id,
                action,
                payload,
            } => self.csms_call(out, now, id, &action, payload),
            Frame::CallResult { id, payload } => match self.outbox.resolve(&id) {
                Some(sent) => self.call_succeeded(out, now, &sent.call, payload),
                None => self.emit(out, now, EventKind::UnknownResponse { id }),
            },
            Frame::CallError { id, code, .. } => match self.outbox.resolve(&id) {
                Some(sent) => {
                    let action = sent.call.action.clone();
                    self.emit(out, now, EventKind::CallFailed { action, code });
                    self.call_failed(out, now, sent);
                }
                None => self.emit(out, now, EventKind::UnknownResponse { id }),
            },
        }
    }

    fn call_succeeded(&mut self, out: &mut Vec<Output>, now: u64, call: &Call, payload: Value) {
        match call.action.as_str() {
            "BootNotification" => self.booted(out, now, payload),
            "Authorize" if call.connector_id == 0 => self.held_reply(out, now, payload),
            "StopTransaction" if call.connector_id == 0 => {}
            "Authorize" | "StartTransaction" | "StopTransaction" => {
                self.transaction_reply(out, now, call, payload)
            }
            _ => {}
        }
    }

    fn call_failed(&mut self, out: &mut Vec<Output>, now: u64, sent: Sent) {
        let Some(sent) = self.retry_transaction_message(out, now, sent) else {
            return;
        };
        let call = &sent.call;
        match call.action.as_str() {
            "BootNotification" => self.boot_failed(now),
            "Authorize" if call.connector_id == 0 => self.held = None,
            "StopTransaction" if call.connector_id == 0 => {}
            "Authorize" | "StartTransaction" | "StopTransaction" => self.transaction_failed(call),
            _ => {}
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

    pub(super) fn reject(
        &self,
        out: &mut Vec<Output>,
        now: u64,
        connector: Option<u32>,
        reason: &str,
    ) {
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
#[path = "mod.test.rs"]
mod tests;
