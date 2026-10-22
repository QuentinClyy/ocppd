use crate::protocol::v201::messages::datatypes::charging_profile::ChargingProfilePurpose;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearChargingProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile_criteria: Option<ClearChargingProfile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearChargingProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile_purpose: Option<ChargingProfilePurpose>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_level: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearChargingProfileResponse {
    pub status: ClearChargingProfileStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearChargingProfileStatus {
    Accepted,
    Unknown,
}
