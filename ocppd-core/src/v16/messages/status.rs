use crate::ci_string::CiString;
use crate::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusNotificationRequest {
    pub connector_id: u32,
    pub error_code: ChargePointErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<CiString<50>>,
    pub status: ChargePointStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<CiString<255>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_error_code: Option<CiString<50>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusNotificationResponse {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargePointErrorCode {
    ConnectorLockFailure,
    #[serde(rename = "EVCommunicationError")]
    EvCommunicationError,
    GroundFailure,
    HighTemperature,
    InternalError,
    LocalListConflict,
    NoError,
    OtherError,
    OverCurrentFailure,
    PowerMeterFailure,
    PowerSwitchFailure,
    ReaderFailure,
    ResetFailure,
    UnderVoltage,
    OverVoltage,
    WeakSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargePointStatus {
    Available,
    Preparing,
    Charging,
    #[serde(rename = "SuspendedEVSE")]
    SuspendedEvse,
    #[serde(rename = "SuspendedEV")]
    SuspendedEv,
    Finishing,
    Reserved,
    Unavailable,
    Faulted,
}

#[cfg(test)]
#[path = "status.test.rs"]
mod tests;
