use crate::wire::ci_string::CiString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse: Option<Evse>,
    pub name: CiString<50>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<CiString<50>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evse {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub name: CiString<50>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<CiString<50>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttributeType {
    #[default]
    Actual,
    Target,
    MinSet,
    MaxSet,
}
