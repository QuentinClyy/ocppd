use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::id_token::MessageFormat;
use crate::protocol::v201::messages::display::{
    ClearDisplayMessageRequest, ClearDisplayMessageResponse, DisplayMessageStatus,
    GetDisplayMessagesRequest, GetDisplayMessagesResponse, KnownStatus, MessageInfo,
    MessagePriority, NotifyDisplayMessagesRequest, SetDisplayMessageRequest,
    SetDisplayMessageResponse,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

pub const DISPLAY_MESSAGE_CTRLR: &str = "DisplayMessageCtrlr";
pub const DISPLAY_MESSAGES: &str = "DisplayMessages";
pub const SUPPORTED_FORMATS: &str = "SupportedFormats";
pub const MAX_DISPLAY_MESSAGES: usize = 10;

impl ChargingStation {
    pub(super) fn set_display_message(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: SetDisplayMessageRequest,
    ) -> SetDisplayMessageResponse {
        let message = request.message;
        if let Some(status) = self.display_refusal(&message) {
            return response(status);
        }
        let replaced = |kept: &MessageInfo| {
            kept.id == message.id
                || (message.priority == MessagePriority::AlwaysFront
                    && kept.priority == MessagePriority::AlwaysFront)
        };
        let kept = self.display_messages.iter().filter(|kept| !replaced(kept));
        if kept.count() >= MAX_DISPLAY_MESSAGES {
            return response(DisplayMessageStatus::Rejected);
        }
        self.remove_display_messages(out, now, replaced);
        let kind = EventKind::DisplayMessageSet {
            id: message.id,
            priority: message.priority,
            content: message.message.content.as_str().to_owned(),
        };
        self.display_messages.push(message);
        self.count_display_messages();
        self.emit(out, now, kind);
        response(DisplayMessageStatus::Accepted)
    }

    pub(super) fn get_display_messages(
        &mut self,
        request: GetDisplayMessagesRequest,
    ) -> GetDisplayMessagesResponse {
        let wanted = |message: &&MessageInfo| {
            request
                .id
                .as_ref()
                .is_none_or(|ids| ids.contains(&message.id))
                && request
                    .priority
                    .is_none_or(|priority| priority == message.priority)
                && request
                    .state
                    .is_none_or(|state| Some(state) == message.state)
        };
        let found: Vec<MessageInfo> = self
            .display_messages
            .iter()
            .filter(wanted)
            .cloned()
            .collect();
        let status = match found.is_empty() {
            true => KnownStatus::Unknown,
            false => KnownStatus::Accepted,
        };
        if !found.is_empty() {
            let report = NotifyDisplayMessagesRequest {
                message_info: Some(found),
                request_id: request.request_id,
                tbc: false,
            };
            self.outbox.push("NotifyDisplayMessages", 0, json!(report));
        }
        GetDisplayMessagesResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn clear_display_message(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ClearDisplayMessageRequest,
    ) -> ClearDisplayMessageResponse {
        let known = self
            .display_messages
            .iter()
            .any(|message| message.id == request.id);
        self.remove_display_messages(out, now, |message| message.id == request.id);
        let status = match known {
            true => KnownStatus::Accepted,
            false => KnownStatus::Unknown,
        };
        ClearDisplayMessageResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn display_deadline(&self) -> Option<u64> {
        let ends = self.display_messages.iter();
        ends.filter_map(|message| message.end_date_time.map(|end| end.unix_ms))
            .min()
    }

    pub(super) fn expire_display_messages(&mut self, out: &mut Vec<Output>, now: u64) {
        self.remove_display_messages(out, now, |message| {
            message.end_date_time.is_some_and(|end| end.unix_ms <= now)
        });
    }

    pub(super) fn drop_transaction_messages(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        transaction_id: &CiString<36>,
    ) {
        self.remove_display_messages(out, now, |message| {
            message
                .transaction_id
                .as_ref()
                .is_some_and(|id| id.matches(transaction_id))
        });
    }

    fn display_refusal(&self, message: &MessageInfo) -> Option<DisplayMessageStatus> {
        let format = match message.message.format {
            MessageFormat::Ascii => "ASCII",
            MessageFormat::Html => "HTML",
            MessageFormat::Uri => "URI",
            MessageFormat::Utf8 => "UTF8",
        };
        let supported = self
            .device_model
            .value(DISPLAY_MESSAGE_CTRLR, SUPPORTED_FORMATS)
            .unwrap_or_default();
        if !supported.split(',').any(|supported| supported == format) {
            return Some(DisplayMessageStatus::NotSupportedMessageFormat);
        }
        if message.display.is_some() {
            return Some(DisplayMessageStatus::Rejected);
        }
        let running = |id: &CiString<36>| {
            let mut sessions = self.sessions.iter();
            sessions.any(|session| {
                let transaction = session.transaction.as_ref();
                transaction.is_some_and(|transaction| transaction.id.matches(id))
            })
        };
        let unknown = message
            .transaction_id
            .as_ref()
            .is_some_and(|id| !running(id));
        unknown.then_some(DisplayMessageStatus::UnknownTransaction)
    }

    fn remove_display_messages(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        removed: impl Fn(&MessageInfo) -> bool,
    ) {
        let messages = std::mem::take(&mut self.display_messages).into_iter();
        let (gone, kept): (Vec<MessageInfo>, _) = messages.partition(|message| removed(message));
        self.display_messages = kept;
        if gone.is_empty() {
            return;
        }
        self.count_display_messages();
        for message in gone {
            self.emit(
                out,
                now,
                EventKind::DisplayMessageRemoved { id: message.id },
            );
        }
    }

    fn count_display_messages(&mut self) {
        let count = self.display_messages.len().to_string();
        self.device_model
            .set_value(DISPLAY_MESSAGE_CTRLR, None, DISPLAY_MESSAGES, count);
    }
}

fn response(status: DisplayMessageStatus) -> SetDisplayMessageResponse {
    SetDisplayMessageResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "display.test.rs"]
mod tests;
