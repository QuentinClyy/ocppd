use crate::v16::messages::datatypes::charging_profile::ChargingProfile;
use crate::v16::messages::datatypes::generic_status::GenericStatus;
use crate::v16::messages::datatypes::id_tag::IdToken;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStartTransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<u32>,
    pub id_tag: IdToken,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile: Option<ChargingProfile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteStartTransactionResponse {
    pub status: GenericStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStopTransactionRequest {
    pub transaction_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteStopTransactionResponse {
    pub status: GenericStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResetRequest {
    #[serde(rename = "type")]
    pub kind: ResetType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResetResponse {
    pub status: GenericStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    Hard,
    Soft,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockConnectorRequest {
    pub connector_id: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnlockConnectorResponse {
    pub status: UnlockStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnlockStatus {
    Unlocked,
    UnlockFailed,
    NotSupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAvailabilityRequest {
    pub connector_id: u32,
    #[serde(rename = "type")]
    pub kind: AvailabilityType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeAvailabilityResponse {
    pub status: AvailabilityStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvailabilityType {
    Inoperative,
    Operative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvailabilityStatus {
    Accepted,
    Rejected,
    Scheduled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearCacheRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearCacheResponse {
    pub status: GenericStatus,
}

#[cfg(test)]
#[path = "remote_control.test.rs"]
mod tests;
