use crate::protocol::v201::messages::datatypes::meter_value::MeterValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterValuesRequest {
    pub evse_id: u32,
    pub meter_value: Vec<MeterValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeterValuesResponse {}
