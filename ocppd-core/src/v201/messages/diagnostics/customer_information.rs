use crate::v201::messages::certificates::CertificateHashData;
use crate::v201::messages::datatypes::id_token::IdToken;
use crate::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInformationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_certificate: Option<CertificateHashData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<IdToken>,
    pub request_id: i32,
    pub report: bool,
    pub clear: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_identifier: Option<CiString<64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInformationResponse {
    pub status: CustomerInformationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerInformationStatus {
    Accepted,
    Rejected,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyCustomerInformationRequest {
    pub data: CiString<512>,
    pub tbc: bool,
    pub seq_no: u32,
    pub generated_at: DateTime,
    pub request_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotifyCustomerInformationResponse {}
