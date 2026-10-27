use super::ChargingStation;
use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::wire::frame::{self, Frame};
use crate::wire::outbox::Sent;
use serde_json::Value;

impl ChargingStation {
    pub(super) fn receive(&mut self, out: &mut Vec<Output>, now: u64, text: &str) {
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
                Some(sent) => self.call_succeeded(out, now, &sent, payload),
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

    fn call_succeeded(&mut self, out: &mut Vec<Output>, now: u64, sent: &Sent, payload: Value) {
        match sent.call.action.as_str() {
            "BootNotification" => self.booted(out, now, payload),
            "Authorize" => self.authorize_reply(out, now, &sent.call, payload),
            "TransactionEvent" => self.transaction_reply(out, now, sent, payload),
            "SignCertificate" => self.sign_certificate_reply(out, now, payload),
            _ => {}
        }
    }

    pub(super) fn call_failed(&mut self, out: &mut Vec<Output>, now: u64, sent: Sent) {
        match sent.call.action.as_str() {
            "BootNotification" => self.boot_failed(now),
            "Authorize" => self.authorize_failed(&sent.call),
            "TransactionEvent" => self.retry_transaction_event(out, now, sent),
            "SignCertificate" => self.sign_certificate_failed(),
            _ => {}
        }
    }
}
