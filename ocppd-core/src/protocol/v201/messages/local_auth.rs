use crate::protocol::v201::messages::datatypes::id_token::{IdToken, IdTokenInfo};
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendLocalListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_authorization_list: Option<Vec<AuthorizationData>>,
    pub version_number: i32,
    pub update_type: UpdateType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationData {
    pub id_token: IdToken,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token_info: Option<IdTokenInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateType {
    Differential,
    Full,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendLocalListResponse {
    pub status: SendLocalListStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendLocalListStatus {
    Accepted,
    Failed,
    VersionMismatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetLocalListVersionRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLocalListVersionResponse {
    pub version_number: i32,
}
