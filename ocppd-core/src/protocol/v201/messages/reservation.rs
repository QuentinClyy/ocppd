use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveNowRequest {
    pub id: i32,
    pub expiry_date_time: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_type: Option<ConnectorType>,
    pub id_token: IdToken,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id_token: Option<IdToken>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveNowResponse {
    pub status: ReserveNowStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReserveNowStatus {
    Accepted,
    Faulted,
    Occupied,
    Rejected,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReservationRequest {
    pub reservation_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReservationResponse {
    pub status: CancelReservationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelReservationStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationStatusUpdateRequest {
    pub reservation_id: i32,
    pub reservation_update_status: ReservationUpdateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationUpdateStatus {
    Expired,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorType {
    #[serde(rename = "cCCS1")]
    CCcs1,
    #[serde(rename = "cCCS2")]
    CCcs2,
    #[serde(rename = "cG105")]
    CG105,
    #[serde(rename = "cTesla")]
    CTesla,
    #[serde(rename = "cType1")]
    CType1,
    #[serde(rename = "cType2")]
    CType2,
    #[serde(rename = "s309-1P-16A")]
    S309OnePhase16A,
    #[serde(rename = "s309-1P-32A")]
    S309OnePhase32A,
    #[serde(rename = "s309-3P-16A")]
    S309ThreePhase16A,
    #[serde(rename = "s309-3P-32A")]
    S309ThreePhase32A,
    #[serde(rename = "sBS1361")]
    SBs1361,
    #[serde(rename = "sCEE-7-7")]
    SCee77,
    #[serde(rename = "sType2")]
    SType2,
    #[serde(rename = "sType3")]
    SType3,
    Other1PhMax16A,
    Other1PhOver16A,
    Other3Ph,
    Pan,
    #[serde(rename = "wInductive")]
    WInductive,
    #[serde(rename = "wResonant")]
    WResonant,
    Undetermined,
    Unknown,
}
