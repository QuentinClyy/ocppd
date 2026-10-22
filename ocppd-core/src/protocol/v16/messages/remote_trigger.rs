use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerMessageRequest {
    pub requested_message: MessageTrigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerMessageResponse {
    pub status: TriggerMessageStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageTrigger {
    BootNotification,
    DiagnosticsStatusNotification,
    FirmwareStatusNotification,
    Heartbeat,
    MeterValues,
    StatusNotification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMessageStatus {
    Accepted,
    Rejected,
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedTriggerMessageRequest {
    pub requested_message: ExtendedMessageTrigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtendedTriggerMessageResponse {
    pub status: TriggerMessageStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtendedMessageTrigger {
    BootNotification,
    LogStatusNotification,
    FirmwareStatusNotification,
    Heartbeat,
    MeterValues,
    SignChargePointCertificate,
    StatusNotification,
}

#[cfg(test)]
#[path = "remote_trigger.test.rs"]
mod tests;
