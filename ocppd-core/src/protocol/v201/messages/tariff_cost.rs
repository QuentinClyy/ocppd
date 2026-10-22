use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostUpdatedRequest {
    pub total_cost: f64,
    pub transaction_id: CiString<36>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostUpdatedResponse {}
