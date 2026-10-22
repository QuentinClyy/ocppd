use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedUpdateFirmwareRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u32>,
    pub request_id: i32,
    pub firmware: Firmware,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Firmware {
    pub location: CiString<512>,
    pub retrieve_date_time: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date_time: Option<DateTime>,
    pub signing_certificate: CiString<5500>,
    pub signature: CiString<800>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedUpdateFirmwareResponse {
    pub status: UpdateFirmwareStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateFirmwareStatus {
    Accepted,
    Rejected,
    AcceptedCanceled,
    InvalidCertificate,
    RevokedCertificate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedFirmwareStatusNotificationRequest {
    pub status: SignedFirmwareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedFirmwareStatusNotificationResponse {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignedFirmwareStatus {
    Downloaded,
    DownloadFailed,
    Downloading,
    DownloadScheduled,
    DownloadPaused,
    Idle,
    InstallationFailed,
    Installing,
    Installed,
    InstallRebooting,
    InstallScheduled,
    InstallVerificationFailed,
    InvalidSignature,
    SignatureVerified,
}

#[cfg(test)]
#[path = "signed_firmware.test.rs"]
mod tests;
