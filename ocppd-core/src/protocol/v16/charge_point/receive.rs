use super::ChargePoint;
use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::protocol::v16::event::EventKind;
use crate::wire::frame::{self, Frame};
use crate::wire::outbox::{Call, Sent};
use serde_json::Value;

impl ChargePoint {
    pub(super) fn receive(&mut self, out: &mut Vec<Output>, now: u64, text: &str) {
        let frame = match frame::parse(text) {
            Ok(frame) => frame,
            Err(err) => {
                let reason = err.to_string();
                self.emit(
                    out,
                    now,
                    EventKind::Link(LinkEvent::FrameRejected { reason }),
                );
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
                None => self.emit(out, now, EventKind::Link(LinkEvent::UnknownResponse { id })),
            },
            Frame::CallError { id, code, .. } => match self.outbox.resolve(&id) {
                Some(sent) => {
                    let action = sent.call.action.clone();
                    self.emit(
                        out,
                        now,
                        EventKind::Link(LinkEvent::CallFailed { action, code }),
                    );
                    self.call_failed(out, now, sent);
                }
                None => self.emit(out, now, EventKind::Link(LinkEvent::UnknownResponse { id })),
            },
        }
    }

    fn call_succeeded(&mut self, out: &mut Vec<Output>, now: u64, call: &Call, payload: Value) {
        match call.action.as_str() {
            "BootNotification" => self.booted(out, now, payload),
            "SignCertificate" => self.sign_certificate_reply(out, now, payload),
            "Authorize" if call.connector_id == 0 => self.held_reply(out, now, payload),
            "Authorize" | "StartTransaction" => self.transaction_reply(out, now, call, payload),
            _ => {}
        }
    }

    pub(super) fn call_failed(&mut self, out: &mut Vec<Output>, now: u64, sent: Sent) {
        let Some(sent) = self.retry_transaction_message(out, now, sent) else {
            return;
        };
        let call = &sent.call;
        match call.action.as_str() {
            "BootNotification" => self.boot_failed(now),
            "Authorize" if call.connector_id == 0 => self.held = None,
            "Authorize" | "StartTransaction" => self.transaction_failed(call),
            _ => {}
        }
    }
}
