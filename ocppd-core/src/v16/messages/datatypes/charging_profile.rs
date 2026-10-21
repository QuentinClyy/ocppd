use crate::wire::date_time::DateTime;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingProfile {
    pub charging_profile_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<i32>,
    pub stack_level: u32,
    pub charging_profile_purpose: ChargingProfilePurpose,
    pub charging_profile_kind: ChargingProfileKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrency_kind: Option<RecurrencyKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime>,
    pub charging_schedule: ChargingSchedule,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_schedule: Option<DateTime>,
    pub charging_rate_unit: ChargingRateUnit,
    pub charging_schedule_period: Vec<ChargingSchedulePeriod>,
    #[serde(
        default,
        deserialize_with = "optional_tenths",
        skip_serializing_if = "Option::is_none"
    )]
    pub min_charging_rate: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSchedulePeriod {
    pub start_period: u32,
    #[serde(deserialize_with = "tenths")]
    pub limit: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_phases: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingProfilePurpose {
    ChargePointMaxProfile,
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
    A,
    W,
}

pub fn tenths_down(rate: f64) -> f64 {
    (rate * 10.0 + 1e-9).floor() / 10.0
}

fn is_tenths(rate: f64) -> bool {
    let scaled = rate * 10.0;
    (scaled - scaled.round()).abs() < 1e-6
}

fn tenths<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    let rate = f64::deserialize(deserializer)?;
    match is_tenths(rate) {
        true => Ok(rate),
        false => Err(D::Error::invalid_value(
            Unexpected::Float(rate),
            &"a multiple of 0.1",
        )),
    }
}

fn optional_tenths<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
    tenths(deserializer).map(Some)
}
