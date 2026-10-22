use crate::protocol::v201::messages::datatypes::component::{Component, Variable};
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariableMonitoringRequest {
    pub set_monitoring_data: Vec<SetMonitoringData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMonitoringData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(default)]
    pub transaction: bool,
    pub value: f64,
    #[serde(rename = "type")]
    pub kind: MonitorType,
    pub severity: u32,
    pub component: Component,
    pub variable: Variable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariableMonitoringResponse {
    pub set_monitoring_result: Vec<SetMonitoringResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMonitoringResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
    pub status: SetMonitoringStatus,
    #[serde(rename = "type")]
    pub kind: MonitorType,
    pub component: Component,
    pub variable: Variable,
    pub severity: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorType {
    UpperThreshold,
    LowerThreshold,
    Delta,
    Periodic,
    PeriodicClockAligned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetMonitoringStatus {
    Accepted,
    UnknownComponent,
    UnknownVariable,
    UnsupportedMonitorType,
    Rejected,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearVariableMonitoringRequest {
    pub id: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearVariableMonitoringResponse {
    pub clear_monitoring_result: Vec<ClearMonitoringResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearMonitoringResult {
    pub status: ClearMonitoringStatus,
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearMonitoringStatus {
    Accepted,
    Rejected,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyEventRequest {
    pub generated_at: DateTime,
    pub tbc: bool,
    pub seq_no: u32,
    pub event_data: Vec<EventData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventData {
    pub event_id: i32,
    pub timestamp: DateTime,
    pub trigger: EventTrigger,
    pub actual_value: CiString<2500>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<bool>,
    pub component: Component,
    pub variable_monitoring_id: i32,
    pub event_notification_type: EventNotificationType,
    pub variable: Variable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventTrigger {
    Alerting,
    Delta,
    Periodic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventNotificationType {
    HardWiredNotification,
    HardWiredMonitor,
    PreconfiguredMonitor,
    CustomMonitor,
}
