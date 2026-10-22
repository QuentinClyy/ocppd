pub mod clear;
pub mod report;

use crate::protocol::v201::messages::datatypes::charging_profile::ChargingProfile;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetChargingProfileRequest {
    pub evse_id: u32,
    pub charging_profile: ChargingProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetChargingProfileResponse {
    pub status: ChargingProfileStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingProfileStatus {
    Accepted,
    Rejected,
}
