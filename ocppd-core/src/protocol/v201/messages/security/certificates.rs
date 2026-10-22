use crate::protocol::v201::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallCertificateRequest {
    pub certificate_type: InstallCertificateUse,
    pub certificate: CiString<5500>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallCertificateUse {
    #[serde(rename = "V2GRootCertificate")]
    V2gRootCertificate,
    #[serde(rename = "MORootCertificate")]
    MoRootCertificate,
    #[serde(rename = "CSMSRootCertificate")]
    CsmsRootCertificate,
    ManufacturerRootCertificate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallCertificateResponse {
    pub status: InstallCertificateStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallCertificateStatus {
    Accepted,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstalledCertificateIdsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_type: Option<Vec<GetCertificateIdUse>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetCertificateIdUse {
    #[serde(rename = "V2GRootCertificate")]
    V2gRootCertificate,
    #[serde(rename = "MORootCertificate")]
    MoRootCertificate,
    #[serde(rename = "CSMSRootCertificate")]
    CsmsRootCertificate,
    #[serde(rename = "V2GCertificateChain")]
    V2gCertificateChain,
    ManufacturerRootCertificate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstalledCertificateIdsResponse {
    pub status: GetInstalledCertificateStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_hash_data_chain: Option<Vec<CertificateHashDataChain>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetInstalledCertificateStatus {
    Accepted,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateHashDataChain {
    pub certificate_hash_data: CertificateHashData,
    pub certificate_type: GetCertificateIdUse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_certificate_hash_data: Option<Vec<CertificateHashData>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateHashData {
    pub hash_algorithm: HashAlgorithm,
    pub issuer_name_hash: CiString<128>,
    pub issuer_key_hash: CiString<128>,
    pub serial_number: CiString<40>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    #[serde(rename = "SHA256")]
    Sha256,
    #[serde(rename = "SHA384")]
    Sha384,
    #[serde(rename = "SHA512")]
    Sha512,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCertificateRequest {
    pub certificate_hash_data: CertificateHashData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCertificateResponse {
    pub status: DeleteCertificateStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteCertificateStatus {
    Accepted,
    Failed,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignCertificateRequest {
    pub csr: CiString<5500>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_type: Option<CertificateSigningUse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateSigningUse {
    ChargingStationCertificate,
    #[serde(rename = "V2GCertificate")]
    V2gCertificate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignCertificateResponse {
    pub status: GenericStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateSignedRequest {
    pub certificate_chain: CiString<10000>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_type: Option<CertificateSigningUse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateSignedResponse {
    pub status: CertificateSignedStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateSignedStatus {
    Accepted,
    Rejected,
}
