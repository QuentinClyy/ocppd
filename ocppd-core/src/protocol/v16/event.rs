use crate::api::event::LinkEvent;
use crate::protocol::v16::messages::core::boot::RegistrationStatus;
use crate::protocol::v16::messages::core::remote_control::ResetType;
use crate::protocol::v16::messages::core::status::ChargePointStatus;
use crate::protocol::v16::messages::core::transaction::Reason;
use crate::protocol::v16::messages::datatypes::id_tag::AuthorizationStatus;
use crate::protocol::v16::messages::firmware::DiagnosticsStatus;
use crate::protocol::v16::messages::security::certificates::CertificateUse;
use crate::protocol::v16::messages::security::log::UploadLogStatus;
use crate::protocol::v16::messages::security::signed_firmware::SignedFirmwareStatus;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    BootStatus {
        status: RegistrationStatus,
    },
    StatusChanged {
        connector: u32,
        status: ChargePointStatus,
    },
    Authorization {
        #[serde(skip_serializing_if = "Option::is_none")]
        connector: Option<u32>,
        id_tag: String,
        status: AuthorizationStatus,
    },
    TransactionStarted {
        connector: u32,
        transaction_id: i32,
    },
    TransactionStopped {
        connector: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        transaction_id: Option<i32>,
        reason: Reason,
    },
    MeterSample {
        connector: u32,
        register_wh: f64,
        power_w: f64,
        soc: f64,
    },
    ConfigurationChanged {
        key: String,
        value: String,
    },
    AvailabilityChanged {
        connector: u32,
        operative: bool,
    },
    Reset {
        reset_type: ResetType,
    },
    AuthorizationExpired {
        id_tag: String,
    },
    LocalListUpdated {
        version: i32,
        entries: usize,
    },
    ChargingProfileSet {
        connector: u32,
        charging_profile_id: i32,
    },
    LogStatus {
        status: UploadLogStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<i32>,
    },
    CertificateInstalled {
        certificate_type: CertificateUse,
    },
    CertificateDeleted {
        certificate_type: CertificateUse,
    },
    ChargePointCertificateInstalled,
    SecurityEvent {
        #[serde(rename = "type")]
        event_type: String,
    },
    FirmwareStatus {
        status: SignedFirmwareStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<i32>,
    },
    DiagnosticsStatus {
        status: DiagnosticsStatus,
    },
    ChargingLimitChanged {
        connector: u32,
        limit_w: f64,
    },
    ChargingProfilesCleared {
        count: usize,
    },
    Reserved {
        connector: u32,
        reservation_id: i32,
        id_tag: String,
        expires: u64,
    },
    ReservationEnded {
        connector: u32,
        reservation_id: i32,
        reason: ReservationEnd,
    },
    RemoteStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        connector: Option<u32>,
        id_tag: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expires: Option<u64>,
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
}
