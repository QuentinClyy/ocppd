use crate::api::event::LinkEvent;
use crate::protocol::v201::messages::availability::ConnectorStatus;
use crate::protocol::v201::messages::datatypes::id_token::AuthorizationStatus;
use crate::protocol::v201::messages::diagnostics::log::UploadLogStatus;
use crate::protocol::v201::messages::firmware::FirmwareStatus;
use crate::protocol::v201::messages::provisioning::boot::RegistrationStatus;
use crate::protocol::v201::messages::provisioning::reset::ResetType;
use crate::protocol::v201::messages::security::certificates::InstallCertificateUse;
use crate::protocol::v201::messages::transactions::{TransactionEventType, TriggerReason};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    BootStatus {
        status: RegistrationStatus,
    },
    StatusChanged {
        evse: u32,
        connector: u32,
        status: ConnectorStatus,
    },
    VariableChanged {
        component: String,
        variable: String,
        value: String,
    },
    Reset {
        reset_type: ResetType,
    },
    Authorization {
        evse: u32,
        id_tag: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<AuthorizationStatus>,
    },
    TransactionEvent {
        evse: u32,
        transaction_id: String,
        event_type: TransactionEventType,
        trigger_reason: TriggerReason,
    },
    RemoteStart {
        evse: u32,
        id_tag: String,
        remote_start_id: i32,
    },
    ConnectorUnlocked {
        evse: u32,
    },
    AvailabilityChanged {
        #[serde(skip_serializing_if = "Option::is_none")]
        evse: Option<u32>,
        operative: bool,
    },
    LocalListUpdated {
        version: i32,
        entries: usize,
    },
    CertificateInstalled {
        certificate_type: InstallCertificateUse,
    },
    CertificateDeleted {
        certificate_type: InstallCertificateUse,
    },
    FirmwareStatus {
        status: FirmwareStatus,
        request_id: i32,
    },
    LogStatus {
        status: UploadLogStatus,
        request_id: i32,
    },
    CustomerDataCleared {
        id_tag: String,
    },
    ChargingStationCertificateInstalled,
    ChargingProfileSet {
        evse: u32,
        charging_profile_id: i32,
    },
    ChargingLimitChanged {
        evse: u32,
        limit_w: f64,
    },
    ChargingProfilesCleared {
        count: usize,
    },
    Reserved {
        evse: u32,
        reservation_id: i32,
        id_tag: String,
        expires: u64,
    },
    ReservationEnded {
        evse: u32,
        reservation_id: i32,
        reason: ReservationEnd,
    },
    NetworkProfileSet {
        slot: usize,
        security_profile: u64,
    },
    SecurityEvent {
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(untagged)]
    Link(LinkEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservationEnd {
    Used,
    Expired,
    Cancelled,
    Removed,
}
