use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingProfile {
    pub id: i32,
    pub stack_level: u32,
    pub charging_profile_purpose: ChargingProfilePurpose,
    pub charging_profile_kind: ChargingProfileKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrency_kind: Option<RecurrencyKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime>,
    pub charging_schedule: Vec<ChargingSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<CiString<36>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSchedule {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_schedule: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    pub charging_rate_unit: ChargingRateUnit,
    pub charging_schedule_period: Vec<ChargingSchedulePeriod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_charging_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_tariff: Option<SalesTariff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSchedulePeriod {
    pub start_period: u32,
    pub limit: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_phases: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_to_use: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesTariff {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_tariff_description: Option<CiString<32>>,
    #[serde(rename = "numEPriceLevels", skip_serializing_if = "Option::is_none")]
    pub num_e_price_levels: Option<i32>,
    pub sales_tariff_entry: Vec<SalesTariffEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesTariffEntry {
    pub relative_time_interval: RelativeTimeInterval,
    #[serde(rename = "ePriceLevel", skip_serializing_if = "Option::is_none")]
    pub e_price_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumption_cost: Option<Vec<ConsumptionCost>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelativeTimeInterval {
    pub start: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumptionCost {
    pub start_value: f64,
    pub cost: Vec<Cost>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cost {
    pub cost_kind: CostKind,
    pub amount: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_multiplier: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostKind {
    CarbonDioxideEmission,
    RelativePricePercentage,
    RenewableGenerationPercentage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingProfilePurpose {
    ChargingStationExternalConstraints,
    ChargingStationMaxProfile,
    TxDefaultProfile,
    TxProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingProfileKind {
    Absolute,
    Recurring,
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrencyKind {
    Daily,
    Weekly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingRateUnit {
    W,
    A,
}
