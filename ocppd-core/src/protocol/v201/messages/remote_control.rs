use crate::protocol::v201::messages::datatypes::charging_profile::ChargingProfile;
use crate::protocol::v201::messages::datatypes::component::Evse;
use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestStartTransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id_token: Option<IdToken>,
    pub id_token: IdToken,
    pub remote_start_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile: Option<ChargingProfile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestStartTransactionResponse {
    pub status: RequestStartStopStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<CiString<36>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestStopTransactionRequest {
    pub transaction_id: CiString<36>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestStopTransactionResponse {
    pub status: RequestStartStopStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStartStopStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockConnectorRequest {
    pub evse_id: u32,
    pub connector_id: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockConnectorResponse {
    pub status: UnlockStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnlockStatus {
    Unlocked,
    UnlockFailed,
    OngoingAuthorizedTransaction,
    UnknownConnector,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse: Option<Evse>,
    pub requested_message: MessageTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageTrigger {
    BootNotification,
    LogStatusNotification,
    FirmwareStatusNotification,
    Heartbeat,
    MeterValues,
    SignChargingStationCertificate,
    #[serde(rename = "SignV2GCertificate")]
    SignV2gCertificate,
    StatusNotification,
    TransactionEvent,
    SignCombinedCertificate,
    PublishFirmwareStatusNotification,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerMessageResponse {
    pub status: TriggerMessageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMessageStatus {
    Accepted,
    Rejected,
    NotImplemented,
}
