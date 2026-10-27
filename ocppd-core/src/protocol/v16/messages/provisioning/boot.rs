use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootNotificationRequest {
    pub charge_point_vendor: CiString<20>,
    pub charge_point_model: CiString<20>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_point_serial_number: Option<CiString<25>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_box_serial_number: Option<CiString<25>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<CiString<50>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iccid: Option<CiString<20>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imsi: Option<CiString<20>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meter_type: Option<CiString<25>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meter_serial_number: Option<CiString<25>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootNotificationResponse {
    pub status: RegistrationStatus,
    pub current_time: DateTime,
    pub interval: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistrationStatus {
    Accepted,
    Pending,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatResponse {
    pub current_time: DateTime,
}

#[cfg(test)]
#[path = "boot.test.rs"]
mod tests;
