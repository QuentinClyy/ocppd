use crate::protocol::v201::messages::datatypes::charging_profile::{
    ChargingRateUnit, ChargingSchedulePeriod,
};
use crate::protocol::v201::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCompositeScheduleRequest {
    pub duration: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_rate_unit: Option<ChargingRateUnit>,
    pub evse_id: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCompositeScheduleResponse {
    pub status: GenericStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<CompositeSchedule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeSchedule {
    pub charging_schedule_period: Vec<ChargingSchedulePeriod>,
    pub evse_id: u32,
    pub duration: u32,
    pub schedule_start: DateTime,
    pub charging_rate_unit: ChargingRateUnit,
}
