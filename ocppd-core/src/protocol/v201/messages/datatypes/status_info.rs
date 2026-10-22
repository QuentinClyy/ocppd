use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusInfo {
    pub reason_code: CiString<20>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<CiString<512>>,
}
