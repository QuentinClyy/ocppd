use super::monitoring::MonitorType;
use crate::protocol::v201::messages::datatypes::component::{
    Component, ComponentVariable, Variable,
};
use crate::protocol::v201::messages::provisioning::report::GetBaseReportResponse;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMonitoringReportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_variable: Option<Vec<ComponentVariable>>,
    pub request_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring_criteria: Option<Vec<MonitoringCriterion>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitoringCriterion {
    ThresholdMonitoring,
    DeltaMonitoring,
    PeriodicMonitoring,
}

pub type GetMonitoringReportResponse = GetBaseReportResponse;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyMonitoringReportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor: Option<Vec<MonitoringData>>,
    pub request_id: i32,
    pub tbc: bool,
    pub seq_no: u32,
    pub generated_at: DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoringData {
    pub component: Component,
    pub variable: Variable,
    pub variable_monitoring: Vec<VariableMonitoring>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableMonitoring {
    pub id: i32,
    pub transaction: bool,
    pub value: f64,
    #[serde(rename = "type")]
    pub kind: MonitorType,
    pub severity: u32,
}
