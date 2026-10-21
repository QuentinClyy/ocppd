use crate::v16;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Event {
    pub station: String,
    pub at: u64,
    #[serde(flatten)]
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Kind {
    V16(v16::event::EventKind),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LinkEvent {
    Connected,
    Disconnected,
    CallTimedOut {
        action: String,
    },
    CallFailed {
        action: String,
        code: String,
    },
    InvalidPayload {
        action: String,
        reason: String,
    },
    FrameRejected {
        reason: String,
    },
    UnknownResponse {
        id: String,
    },
    IntentRejected {
        #[serde(skip_serializing_if = "Option::is_none")]
        connector: Option<u32>,
        reason: String,
    },
    CallRetried {
        action: String,
        attempt: u32,
        retry_at: u64,
    },
    CallAbandoned {
        action: String,
    },
}

#[cfg(test)]
#[path = "event.test.rs"]
mod tests;
