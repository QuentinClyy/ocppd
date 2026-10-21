use crate::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootNotificationRequest {
    pub charging_station: ChargingStationInfo,
    pub reason: BootReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingStationInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<CiString<25>>,
    pub model: CiString<20>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modem: Option<Modem>,
    pub vendor_name: CiString<50>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<CiString<50>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Modem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iccid: Option<CiString<20>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imsi: Option<CiString<20>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootReason {
    ApplicationReset,
    FirmwareUpdate,
    LocalReset,
    PowerUp,
    RemoteReset,
    ScheduledReset,
    Triggered,
    Unknown,
    Watchdog,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootNotificationResponse {
    pub current_time: DateTime,
    pub interval: u32,
    pub status: RegistrationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistrationStatus {
    Accepted,
    Pending,
    Rejected,
}
