use crate::v16::messages::core::boot::RegistrationStatus;
use crate::v16::messages::core::remote_control::ResetType;
use crate::v16::messages::core::status::ChargePointStatus;
use crate::v16::messages::core::transaction::Reason;
use crate::v16::messages::datatypes::id_tag::AuthorizationStatus;
use serde::Serialize;

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
        #[serde(skip_serializing_if = "Option::is_none")]
        connector: Option<u32>,
        reason: String,
    },
    StatusChanged {
        connector: u32,
        status: ChargePointStatus,
    },
    Authorization {
        #[serde(skip_serializing_if = "Option::is_none")]
        connector: Option<u32>,
        id_tag: String,
        status: AuthorizationStatus,
    },
    TransactionStarted {
        connector: u32,
        transaction_id: i32,
    },
    TransactionStopped {
        connector: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        transaction_id: Option<i32>,
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
    AuthorizationExpired {
        id_tag: String,
    },
    CallRetried {
        action: String,
        attempt: u32,
        retry_at: u64,
    },
    CallAbandoned {
        action: String,
    },
    LocalListUpdated {
        version: i32,
        entries: usize,
    },
    ChargingProfileSet {
        connector: u32,
        charging_profile_id: i32,
    },
    ChargingProfilesCleared {
        count: usize,
    },
    Reserved {
        connector: u32,
        reservation_id: i32,
        id_tag: String,
        expires: u64,
    },
    ReservationEnded {
        connector: u32,
        reservation_id: i32,
        reason: ReservationEnd,
    },
    RemoteStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        connector: Option<u32>,
        id_tag: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expires: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservationEnd {
    Used,
    Expired,
    Cancelled,
}
