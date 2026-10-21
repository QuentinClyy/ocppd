use crate::v201::messages::datatypes::status_info::StatusInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetRequest {
    #[serde(rename = "type")]
    pub kind: ResetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    Immediate,
    OnIdle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetResponse {
    pub status: ResetStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetStatus {
    Accepted,
    Rejected,
    Scheduled,
}
