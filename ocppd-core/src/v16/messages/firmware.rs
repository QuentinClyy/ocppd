use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use crate::wire::uri::Uri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFirmwareRequest {
    pub location: Uri,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    pub retrieve_date: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFirmwareResponse {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareStatusNotificationRequest {
    pub status: FirmwareStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareStatusNotificationResponse {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirmwareStatus {
    Downloaded,
    DownloadFailed,
    Downloading,
    Idle,
    InstallationFailed,
    Installing,
    Installed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagnosticsRequest {
    pub location: Uri,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_time: Option<DateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagnosticsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<CiString<255>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsStatusNotificationRequest {
    pub status: DiagnosticsStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsStatusNotificationResponse {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticsStatus {
    Idle,
    Uploaded,
    UploadFailed,
    Uploading,
}

#[cfg(test)]
#[path = "firmware.test.rs"]
mod tests;
