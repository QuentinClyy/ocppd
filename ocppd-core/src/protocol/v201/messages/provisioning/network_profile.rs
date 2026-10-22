use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNetworkProfileRequest {
    pub configuration_slot: i32,
    pub connection_data: NetworkConnectionProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConnectionProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apn: Option<Apn>,
    pub ocpp_version: OcppVersion,
    pub ocpp_transport: OcppTransport,
    pub ocpp_csms_url: CiString<512>,
    pub message_timeout: i32,
    pub security_profile: i32,
    pub ocpp_interface: OcppInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpn: Option<Vpn>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apn {
    pub apn: CiString<512>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apn_user_name: Option<CiString<20>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apn_password: Option<CiString<20>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_pin: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_network: Option<CiString<6>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_only_preferred_network: Option<bool>,
    pub apn_authentication: ApnAuthentication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApnAuthentication {
    #[serde(rename = "CHAP")]
    Chap,
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "PAP")]
    Pap,
    #[serde(rename = "AUTO")]
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vpn {
    pub server: CiString<512>,
    pub user: CiString<20>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<CiString<20>>,
    pub password: CiString<20>,
    pub key: CiString<255>,
    #[serde(rename = "type")]
    pub kind: VpnType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VpnType {
    #[serde(rename = "IKEv2")]
    IkeV2,
    #[serde(rename = "IPSec")]
    IpSec,
    #[serde(rename = "L2TP")]
    L2tp,
    #[serde(rename = "PPTP")]
    Pptp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcppVersion {
    #[serde(rename = "OCPP12")]
    Ocpp12,
    #[serde(rename = "OCPP15")]
    Ocpp15,
    #[serde(rename = "OCPP16")]
    Ocpp16,
    #[serde(rename = "OCPP20")]
    Ocpp20,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcppTransport {
    #[serde(rename = "JSON")]
    Json,
    #[serde(rename = "SOAP")]
    Soap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcppInterface {
    Wired0,
    Wired1,
    Wired2,
    Wired3,
    Wireless0,
    Wireless1,
    Wireless2,
    Wireless3,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNetworkProfileResponse {
    pub status: SetNetworkProfileStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetNetworkProfileStatus {
    Accepted,
    Rejected,
    Failed,
}
