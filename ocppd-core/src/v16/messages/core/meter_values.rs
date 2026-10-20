use crate::v16::messages::datatypes::meter_value::MeterValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterValuesRequest {
    pub connector_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<i32>,
    pub meter_value: Vec<MeterValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeterValuesResponse {}

#[cfg(test)]
#[path = "meter_values.test.rs"]
mod tests;
