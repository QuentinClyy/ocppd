use crate::v16::messages::datatypes::generic_status::GenericStatus;
use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignCertificateRequest {
    pub csr: CiString<5500>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignCertificateResponse {
    pub status: GenericStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateSignedRequest {
    pub certificate_chain: CiString<10000>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificateSignedResponse {
    pub status: GenericStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallCertificateRequest {
    pub certificate_type: CertificateUse,
    pub certificate: CiString<5500>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallCertificateResponse {
    pub status: InstallCertificateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallCertificateStatus {
    Accepted,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstalledCertificateIdsRequest {
    pub certificate_type: CertificateUse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstalledCertificateIdsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_hash_data: Option<Vec<CertificateHashData>>,
    pub status: GetInstalledCertificateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetInstalledCertificateStatus {
    Accepted,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCertificateRequest {
    pub certificate_hash_data: CertificateHashData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteCertificateResponse {
    pub status: DeleteCertificateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteCertificateStatus {
    Accepted,
    Failed,
    NotFound,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateUse {
    CentralSystemRootCertificate,
    ManufacturerRootCertificate,
}

#[cfg(test)]
#[path = "certificates.test.rs"]
mod tests;
