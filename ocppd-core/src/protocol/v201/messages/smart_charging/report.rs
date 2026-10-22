use crate::protocol::v201::messages::datatypes::charging_profile::{
    ChargingProfile, ChargingProfilePurpose,
};
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChargingProfilesRequest {
    pub request_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<u32>,
    pub charging_profile: ChargingProfileCriterion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingProfileCriterion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile_purpose: Option<ChargingProfilePurpose>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile_id: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_limit_source: Option<Vec<ChargingLimitSource>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChargingProfilesResponse {
    pub status: GetChargingProfileStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetChargingProfileStatus {
    Accepted,
    NoProfiles,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportChargingProfilesRequest {
    pub request_id: i32,
    pub charging_limit_source: ChargingLimitSource,
    pub charging_profile: Vec<ChargingProfile>,
    pub tbc: bool,
    pub evse_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingLimitSource {
    #[serde(rename = "EMS")]
    Ems,
    Other,
    #[serde(rename = "SO")]
    So,
    #[serde(rename = "CSO")]
    Cso,
}
