use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLogRequest {
    pub log: LogParameters,
    pub log_type: LogType,
    pub request_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogParameters {
    pub remote_location: CiString<512>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_timestamp: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_timestamp: Option<DateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogType {
    DiagnosticsLog,
    SecurityLog,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLogResponse {
    pub status: LogStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<CiString<255>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogStatus {
    Accepted,
    Rejected,
    AcceptedCanceled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogStatusNotificationRequest {
    pub status: UploadLogStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogStatusNotificationResponse {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UploadLogStatus {
    BadMessage,
    Idle,
    NotSupportedOperation,
    PermissionDenied,
    Uploaded,
    UploadFailure,
    Uploading,
    AcceptedCanceled,
}
