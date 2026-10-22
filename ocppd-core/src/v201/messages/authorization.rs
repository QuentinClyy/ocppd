use crate::v201::messages::certificates::HashAlgorithm;
use crate::v201::messages::datatypes::id_token::{IdToken, IdTokenInfo};
use crate::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeRequest {
    pub id_token: IdToken,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<CiString<5500>>,
    #[serde(
        rename = "iso15118CertificateHashData",
        skip_serializing_if = "Option::is_none"
    )]
    pub iso15118_certificate_hash_data: Option<Vec<OcspRequestData>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcspRequestData {
    pub hash_algorithm: HashAlgorithm,
    pub issuer_name_hash: CiString<128>,
    pub issuer_key_hash: CiString<128>,
    pub serial_number: CiString<40>,
    #[serde(rename = "responderURL")]
    pub responder_url: CiString<512>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeResponse {
    pub id_token_info: IdTokenInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_status: Option<AuthorizeCertificateStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizeCertificateStatus {
    Accepted,
    SignatureError,
    CertificateExpired,
    CertificateRevoked,
    NoCertificateAvailable,
    CertChainError,
    ContractCancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearCacheRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResponse {
    pub status: ClearCacheStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearCacheStatus {
    Accepted,
    Rejected,
}
