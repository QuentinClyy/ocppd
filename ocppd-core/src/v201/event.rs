use crate::api::event::LinkEvent;
use crate::v201::messages::availability::ConnectorStatus;
use crate::v201::messages::certificates::InstallCertificateUse;
use crate::v201::messages::datatypes::id_token::AuthorizationStatus;
use crate::v201::messages::diagnostics::log::UploadLogStatus;
use crate::v201::messages::firmware::FirmwareStatus;
use crate::v201::messages::provisioning::boot::RegistrationStatus;
use crate::v201::messages::provisioning::reset::ResetType;
use crate::v201::messages::transactions::{TransactionEventType, TriggerReason};
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
    SecurityEvent {
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(untagged)]
    Link(LinkEvent),
}
