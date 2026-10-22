mod availability;
mod csms_call;
mod device_model;
mod driver;
mod metering;
mod registration;
mod remote_control;
mod report;
mod reset;
mod status;
mod transaction;
mod trigger;
mod variables;

use crate::api::event::{Event, Kind, LinkEvent};
use crate::api::io::{Input, Output};
use crate::station::Station;
use crate::v201::event::EventKind;
use crate::v201::messages::availability::ConnectorStatus;
use crate::v201::messages::custom_data::without_custom_data;
use crate::v201::messages::provisioning::boot::{
    BootNotificationRequest, BootReason, ChargingStationInfo,
};
use crate::wire::ci_string::{CiString, TooLong};
use crate::wire::frame::{self, Frame};
use crate::wire::outbox::{Call, Outbox, Sent};
use crate::wire::payload;
use availability::Availability;
use device_model::{DeviceModel, table};
use registration::Registration;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use transaction::Session;

const TRANSACTION_MESSAGES: [&str; 1] = ["TransactionEvent"];

pub struct ChargingStation {
    station: Station,
    outbox: Outbox,
    reported: Vec<Option<ConnectorStatus>>,
    resend_statuses: bool,
    offline_since: Option<u64>,
    sessions: Vec<Session>,
    evses: Vec<Availability>,
    station_availability: Availability,
    device_model: DeviceModel,
    resetting: bool,
    next_aligned: Option<u64>,
    boot_request: BootNotificationRequest,
    registration: Registration,
}

impl ChargingStation {
    pub fn new(station: Station) -> Result<Self, TooLong> {
        let boot_request = BootNotificationRequest {
            charging_station: ChargingStationInfo {
                serial_number: Some(CiString::new(&station.serial_number)?),
                model: CiString::new(&station.model)?,
                modem: None,
                vendor_name: CiString::new(&station.vendor)?,
                firmware_version: Some(CiString::new(&station.firmware_version)?),
            },
            reason: BootReason::PowerUp,
        };
        let connectors = station.connectors.len();
        let device_model = DeviceModel::new(table::entries(&station)?);
        Ok(ChargingStation {
            station,
            outbox: Outbox::new(&TRANSACTION_MESSAGES),
            reported: vec![None; connectors],
            resend_statuses: false,
            offline_since: None,
            sessions: (0..connectors).map(|_| Session::default()).collect(),
            evses: vec![Availability::default(); connectors],
            station_availability: Availability::default(),
            device_model,
            resetting: false,
            next_aligned: None,
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
            self.sampling_deadline(),
            self.next_aligned,
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub fn handle(&mut self, now: u64, input: Input) -> Vec<Output> {
        let mut out = Vec::new();
        match input {
            Input::Connected => {
                self.emit(&mut out, now, EventKind::Link(LinkEvent::Connected));
                self.connected(now);
                self.back_online(now);
            }
            Input::Disconnected => {
                self.disconnected();
                self.outbox
                    .keep(|action, _| TRANSACTION_MESSAGES.contains(&action));
                self.mark_queued_offline();
                self.offline_since = Some(now);
                self.emit(&mut out, now, EventKind::Link(LinkEvent::Disconnected));
            }
            Input::Tick => self.tick(&mut out, now),
            Input::Frame(text) => self.receive(&mut out, now, &text),
            Input::Intent(intent) => self.intent(&mut out, now, intent),
        }
        if self.is_accepted() {
            self.report_statuses(&mut out, now);
            self.aligned_tick(&mut out, now);
        }
        if self.reset_due(now) {
            self.rebooted();
            out.push(Output::Close);
        }
        if self.is_connected()
            && let Some(frame) = self.outbox.next_frame(now, self.is_accepted())
        {
            out.push(Output::Send(frame));
        }
        out
    }

    fn tick(&mut self, out: &mut Vec<Output>, now: u64) {
        if let Some(sent) = self.outbox.expire(now) {
            let action = sent.call.action.clone();
            let kind = LinkEvent::CallTimedOut { action };
            self.emit(out, now, EventKind::Link(kind));
            self.call_failed(out, now, sent);
        }
        self.registration_tick(now);
        self.transaction_tick(out, now);
    }

    fn receive(&mut self, out: &mut Vec<Output>, now: u64, text: &str) {
        let frame = match frame::parse(text) {
            Ok(frame) => frame,
            Err(err) => {
                let reason = err.to_string();
                let kind = LinkEvent::FrameRejected { reason };
                return self.emit(out, now, EventKind::Link(kind));
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
                None => self.emit(out, now, EventKind::Link(LinkEvent::UnknownResponse { id })),
            },
            Frame::CallError { id, code, .. } => match self.outbox.resolve(&id) {
                Some(sent) => {
                    let action = sent.call.action.clone();
                    let kind = LinkEvent::CallFailed { action, code };
                    self.emit(out, now, EventKind::Link(kind));
                    self.call_failed(out, now, sent);
                }
                None => self.emit(out, now, EventKind::Link(LinkEvent::UnknownResponse { id })),
            },
        }
    }

    fn call_succeeded(&mut self, out: &mut Vec<Output>, now: u64, call: &Call, payload: Value) {
        match call.action.as_str() {
            "BootNotification" => self.booted(out, now, payload),
            "Authorize" => self.authorize_reply(out, now, call, payload),
            "TransactionEvent" => self.transaction_reply(out, now, call, payload),
            _ => {}
        }
    }

    fn call_failed(&mut self, out: &mut Vec<Output>, now: u64, sent: Sent) {
        match sent.call.action.as_str() {
            "BootNotification" => self.boot_failed(now),
            "Authorize" => self.authorize_failed(&sent.call),
            "TransactionEvent" => self.retry_transaction_event(out, now, sent),
            _ => {}
        }
    }

    fn decode<T: DeserializeOwned + Serialize>(
        &self,
        out: &mut Vec<Output>,
        now: u64,
        action: &str,
        payload: Value,
    ) -> Option<T> {
        match payload::decode(without_custom_data(payload)) {
            Ok(decoded) => Some(decoded),
            Err(invalid) => {
                let action = action.to_owned();
                let reason = invalid.reason;
                let kind = LinkEvent::InvalidPayload { action, reason };
                self.emit(out, now, EventKind::Link(kind));
                None
            }
        }
    }

    fn reject(&self, out: &mut Vec<Output>, now: u64, connector: Option<u32>, reason: &str) {
        let reason = reason.to_owned();
        let kind = LinkEvent::IntentRejected { connector, reason };
        self.emit(out, now, EventKind::Link(kind));
    }

    fn emit(&self, out: &mut Vec<Output>, now: u64, kind: EventKind) {
        out.push(Output::Emit(Event {
            station: self.station.id.clone(),
            at: now,
            kind: Kind::V201(kind),
        }));
    }
}
