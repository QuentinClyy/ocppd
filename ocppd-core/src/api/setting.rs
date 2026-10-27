use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Setting {
    pub key: String,
    pub readonly: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
