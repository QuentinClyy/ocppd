use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdToken {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<Vec<AdditionalInfo>>,
    pub id_token: CiString<36>,
    #[serde(rename = "type")]
    pub kind: IdTokenType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalInfo {
    pub additional_id_token: CiString<36>,
    #[serde(rename = "type")]
    pub kind: CiString<50>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdTokenType {
    Central,
    #[serde(rename = "eMAID")]
    EMaid,
    #[serde(rename = "ISO14443")]
    Iso14443,
    #[serde(rename = "ISO15693")]
    Iso15693,
    KeyCode,
    Local,
    MacAddress,
    NoAuthorization,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdTokenInfo {
    pub status: AuthorizationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_expiry_date_time: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language1: Option<CiString<8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse_id: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id_token: Option<IdToken>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language2: Option<CiString<8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_message: Option<MessageContent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationStatus {
    Accepted,
    Blocked,
    ConcurrentTx,
    Expired,
    Invalid,
    NoCredit,
    NotAllowedTypeEVSE,
    NotAtThisLocation,
    NotAtThisTime,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageContent {
    pub format: MessageFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<CiString<8>>,
    pub content: CiString<512>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageFormat {
    #[serde(rename = "ASCII")]
    Ascii,
    #[serde(rename = "HTML")]
    Html,
    #[serde(rename = "URI")]
    Uri,
    #[serde(rename = "UTF8")]
    Utf8,
}
