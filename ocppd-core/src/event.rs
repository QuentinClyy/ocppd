use crate::frame::Frame;
use crate::v16::types::RegistrationStatus;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Connected,
    Disconnected,
    Tick,
    Frame(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Send(Frame),
    Emit(Event),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Event {
    pub station: String,
    pub at: u64,
    #[serde(flatten)]
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    Connected,
    Disconnected,
    BootStatus { status: RegistrationStatus },
    CallTimedOut { action: String },
    CallFailed { action: String, code: String },
    InvalidPayload { action: String, reason: String },
    FrameRejected { reason: String },
    UnknownResponse { id: String },
}
