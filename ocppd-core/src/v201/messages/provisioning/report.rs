use crate::v201::messages::datatypes::component::{AttributeType, Component, Variable};
use crate::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBaseReportRequest {
    pub request_id: i32,
    pub report_base: ReportBase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportBase {
    ConfigurationInventory,
    FullInventory,
    SummaryInventory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBaseReportResponse {
    pub status: GenericDeviceModelStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenericDeviceModelStatus {
    Accepted,
    Rejected,
    NotSupported,
    EmptyResultSet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyReportRequest {
    pub request_id: i32,
    pub generated_at: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_data: Option<Vec<ReportData>>,
    pub tbc: bool,
    pub seq_no: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotifyReportResponse {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    pub component: Component,
    pub variable: Variable,
    pub variable_attribute: Vec<VariableAttribute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable_characteristics: Option<VariableCharacteristics>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableAttribute {
    #[serde(rename = "type")]
    pub kind: AttributeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<CiString<2500>>,
    pub mutability: Mutability,
    pub persistent: bool,
    pub constant: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutability {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableCharacteristics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<CiString<16>>,
    pub data_type: DataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values_list: Option<CiString<1000>>,
    pub supports_monitoring: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "decimal")]
    Decimal,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "dateTime")]
    DateTime,
    #[serde(rename = "boolean")]
    Boolean,
    OptionList,
    SequenceList,
    MemberList,
}
