use crate::v201::messages::datatypes::component::Evse;
use crate::v201::messages::datatypes::id_token::{IdToken, IdTokenInfo, MessageContent};
use crate::v201::messages::datatypes::meter_value::MeterValue;
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEventRequest {
    pub event_type: TransactionEventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meter_value: Option<Vec<MeterValue>>,
    pub timestamp: DateTime,
    pub trigger_reason: TriggerReason,
    pub seq_no: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_phases_used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cable_max_current: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_id: Option<i32>,
    pub transaction_info: TransactionInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evse: Option<Evse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<IdToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionEventType {
    Ended,
    Started,
    Updated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerReason {
    Authorized,
    CablePluggedIn,
    ChargingRateChanged,
    ChargingStateChanged,
    Deauthorized,
    EnergyLimitReached,
    #[serde(rename = "EVCommunicationLost")]
    EvCommunicationLost,
    #[serde(rename = "EVConnectTimeout")]
    EvConnectTimeout,
    MeterValueClock,
    MeterValuePeriodic,
    TimeLimitReached,
    Trigger,
    UnlockCommand,
    StopAuthorized,
    #[serde(rename = "EVDeparted")]
    EvDeparted,
    #[serde(rename = "EVDetected")]
    EvDetected,
    RemoteStop,
    RemoteStart,
    AbnormalCondition,
    SignedDataReceived,
    ResetCommand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    pub transaction_id: CiString<36>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_state: Option<ChargingState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent_charging: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_reason: Option<StoppedReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_start_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargingState {
    Charging,
    #[serde(rename = "EVConnected")]
    EvConnected,
    #[serde(rename = "SuspendedEV")]
    SuspendedEv,
    #[serde(rename = "SuspendedEVSE")]
    SuspendedEvse,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoppedReason {
    DeAuthorized,
    EmergencyStop,
    EnergyLimitReached,
    #[serde(rename = "EVDisconnected")]
    EvDisconnected,
    GroundFault,
    ImmediateReset,
    Local,
    LocalOutOfCredit,
    MasterPass,
    Other,
    OvercurrentFault,
    PowerLoss,
    PowerQuality,
    Reboot,
    Remote,
    #[serde(rename = "SOCLimitReached")]
    SocLimitReached,
    #[serde(rename = "StoppedByEV")]
    StoppedByEv,
    TimeLimitReached,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEventResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token_info: Option<IdTokenInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_personal_message: Option<MessageContent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionStatusRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<CiString<36>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionStatusResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ongoing_indicator: Option<bool>,
    pub messages_in_queue: bool,
}
