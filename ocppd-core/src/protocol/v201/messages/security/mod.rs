pub mod certificates;

use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventNotificationRequest {
    #[serde(rename = "type")]
    pub kind: CiString<50>,
    pub timestamp: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_info: Option<CiString<255>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityEventNotificationResponse {}
