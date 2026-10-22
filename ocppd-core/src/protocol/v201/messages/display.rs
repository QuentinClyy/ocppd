use crate::protocol::v201::messages::datatypes::component::Component;
use crate::protocol::v201::messages::datatypes::id_token::MessageContent;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetDisplayMessageRequest {
    pub message: MessageInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Component>,
    pub id: i32,
    pub priority: MessagePriority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MessageState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date_time: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date_time: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<CiString<36>>,
    pub message: MessageContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    AlwaysFront,
    InFront,
    NormalCycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageState {
    Charging,
    Faulted,
    Idle,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDisplayMessageResponse {
    pub status: DisplayMessageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMessageStatus {
    Accepted,
    NotSupportedMessageFormat,
    Rejected,
    NotSupportedPriority,
    NotSupportedState,
    UnknownTransaction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDisplayMessagesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<i32>>,
    pub request_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<MessagePriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MessageState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDisplayMessagesResponse {
    pub status: KnownStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnownStatus {
    Accepted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyDisplayMessagesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_info: Option<Vec<MessageInfo>>,
    pub request_id: i32,
    pub tbc: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearDisplayMessageRequest {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearDisplayMessageResponse {
    pub status: KnownStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_info: Option<StatusInfo>,
}
