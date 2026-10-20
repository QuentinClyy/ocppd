use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

pub type IdToken = CiString<20>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdTagInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id_tag: Option<IdToken>,
    pub status: AuthorizationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationStatus {
    Accepted,
    Blocked,
    Expired,
    Invalid,
    ConcurrentTx,
}
