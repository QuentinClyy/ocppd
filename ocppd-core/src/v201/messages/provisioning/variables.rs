use crate::v201::messages::datatypes::component::{AttributeType, Component, Variable};
use crate::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariablesRequest {
    pub get_variable_data: Vec<GetVariableData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariableData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_type: Option<AttributeType>,
    pub component: Component,
    pub variable: Variable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariablesResponse {
    pub get_variable_result: Vec<GetVariableResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariableResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_status_info: Option<StatusInfo>,
    pub attribute_status: GetVariableStatus,
    pub attribute_type: AttributeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value: Option<CiString<2500>>,
    pub component: Component,
    pub variable: Variable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetVariableStatus {
    Accepted,
    Rejected,
    UnknownComponent,
    UnknownVariable,
    NotSupportedAttributeType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariablesRequest {
    pub set_variable_data: Vec<SetVariableData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariableData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_type: Option<AttributeType>,
    pub attribute_value: CiString<1000>,
    pub component: Component,
    pub variable: Variable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariablesResponse {
    pub set_variable_result: Vec<SetVariableResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariableResult {
    pub attribute_type: AttributeType,
    pub attribute_status: SetVariableStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_status_info: Option<StatusInfo>,
    pub component: Component,
    pub variable: Variable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetVariableStatus {
    Accepted,
    Rejected,
    UnknownComponent,
    UnknownVariable,
    NotSupportedAttributeType,
    RebootRequired,
}
