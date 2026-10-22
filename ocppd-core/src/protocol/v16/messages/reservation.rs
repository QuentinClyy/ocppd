use crate::protocol::v16::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v16::messages::datatypes::id_tag::IdToken;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveNowRequest {
    pub connector_id: u32,
    pub expiry_date: DateTime,
    pub id_tag: IdToken,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id_tag: Option<IdToken>,
    pub reservation_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReserveNowResponse {
    pub status: ReservationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationStatus {
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
pub struct CancelReservationResponse {
    pub status: GenericStatus,
}

#[cfg(test)]
#[path = "reservation.test.rs"]
mod tests;
