use crate::frame::Frame;
use crate::intent::Intent;
use crate::v16::messages::boot::RegistrationStatus;
use crate::v16::messages::id_tag::AuthorizationStatus;
use crate::v16::messages::remote_control::ResetType;
use crate::v16::messages::status::ChargePointStatus;
use crate::v16::messages::transaction::Reason;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Connected,
    Disconnected,
    Tick,
    Frame(String),
    Intent(Intent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Send(Frame),
    Emit(Event),
    Close,
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
    BootStatus {
        status: RegistrationStatus,
    },
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
        connector: u32,
        reason: String,
    },
    StatusChanged {
        connector: u32,
        status: ChargePointStatus,
    },
    Authorization {
        connector: u32,
        id_tag: String,
        status: AuthorizationStatus,
    },
    TransactionStarted {
        connector: u32,
        transaction_id: i32,
    },
    TransactionStopped {
        connector: u32,
        transaction_id: i32,
        reason: Reason,
    },
    MeterSample {
        connector: u32,
        register_wh: f64,
        power_w: f64,
        soc: f64,
    },
    ConfigurationChanged {
        key: String,
        value: String,
    },
    AvailabilityChanged {
        connector: u32,
        operative: bool,
    },
    Reset {
        reset_type: ResetType,
    },
}
