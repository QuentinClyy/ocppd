use super::ChargePoint;
use super::local_auth::id_tags::IdTags;
use super::local_auth::list::Limits;
use super::provisioning::configuration::{
    AUTHORIZATION_KEY, LOCAL_AUTH_LIST_MAX_LENGTH, SECURITY_PROFILE, SEND_LOCAL_LIST_MAX_LENGTH,
    is_derived,
};
use super::security::security_event::RECONFIGURATION_OF_SECURITY_PARAMETERS;
use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::data_transfer::{
    DataTransferRequest, DataTransferResponse, DataTransferStatus,
};
use crate::protocol::v16::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v16::messages::firmware::{
    GetDiagnosticsRequest, UpdateFirmwareRequest, UpdateFirmwareResponse,
};
use crate::protocol::v16::messages::local_auth::{
    GetLocalListVersionRequest, GetLocalListVersionResponse, SendLocalListRequest,
    SendLocalListResponse, UpdateStatus,
};
use crate::protocol::v16::messages::provisioning::configuration::{
    ChangeConfigurationRequest, ChangeConfigurationResponse, ConfigurationStatus,
    GetConfigurationRequest,
};
use crate::protocol::v16::messages::remote_control::{
    ChangeAvailabilityRequest, ChangeAvailabilityResponse, ClearCacheRequest, ClearCacheResponse,
    RemoteStartTransactionRequest, RemoteStartTransactionResponse, RemoteStopTransactionRequest,
    RemoteStopTransactionResponse, ResetRequest, ResetResponse, UnlockConnectorRequest,
    UnlockConnectorResponse,
};
use crate::protocol::v16::messages::remote_trigger::{
    ExtendedTriggerMessageRequest, ExtendedTriggerMessageResponse, TriggerMessageRequest,
    TriggerMessageResponse,
};
use crate::protocol::v16::messages::reservation::{
    CancelReservationRequest, CancelReservationResponse, ReserveNowRequest, ReserveNowResponse,
};
use crate::protocol::v16::messages::security::certificates::{
    CertificateSignedRequest, DeleteCertificateRequest, GetInstalledCertificateIdsRequest,
    InstallCertificateRequest,
};
use crate::protocol::v16::messages::security::log::GetLogRequest;
use crate::protocol::v16::messages::security::signed_firmware::{
    SignedUpdateFirmwareRequest, SignedUpdateFirmwareResponse,
};
use crate::protocol::v16::messages::smart_charging::{
    ClearChargingProfileRequest, ClearChargingProfileResponse, GetCompositeScheduleRequest,
    SetChargingProfileRequest, SetChargingProfileResponse,
};
use crate::wire::ci_string::CiString;
use crate::wire::frame::Frame;
use crate::wire::payload::{self, Violation};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

enum ErrorCode {
    NotImplemented,
    Invalid(Violation),
}

impl ErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::NotImplemented => "NotImplemented",
            ErrorCode::Invalid(Violation::Formation) => "FormationViolation",
            ErrorCode::Invalid(Violation::Occurrence) => "OccurenceConstraintViolation",
            ErrorCode::Invalid(Violation::Type) => "TypeConstraintViolation",
            ErrorCode::Invalid(Violation::Property) => "PropertyConstraintViolation",
        }
    }
}

impl ChargePoint {
    pub(super) fn csms_call(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        id: String,
        action: &str,
        payload: Value,
    ) {
        let answered = match action {
            "GetConfiguration" => answer(payload, |request: GetConfigurationRequest| {
                self.configuration.get(&request)
            }),
            "ChangeConfiguration" => answer(payload, |request: ChangeConfigurationRequest| {
                self.change_configuration(out, now, &request)
            }),
            "RemoteStartTransaction" => {
                answer(payload, |request: RemoteStartTransactionRequest| {
                    RemoteStartTransactionResponse {
                        status: self.remote_start(out, now, request),
                    }
                })
            }
            "RemoteStopTransaction" => answer(payload, |request: RemoteStopTransactionRequest| {
                RemoteStopTransactionResponse {
                    status: self.remote_stop(out, now, request),
                }
            }),
            "UnlockConnector" => answer(payload, |request: UnlockConnectorRequest| {
                UnlockConnectorResponse {
                    status: self.unlock(out, now, request),
                }
            }),
            "Reset" => answer(payload, |request: ResetRequest| ResetResponse {
                status: self.reset(out, now, request),
            }),
            "ChangeAvailability" => answer(payload, |request: ChangeAvailabilityRequest| {
                ChangeAvailabilityResponse {
                    status: self.change_availability(out, now, request),
                }
            }),
            "ClearCache" => answer(payload, |_: ClearCacheRequest| {
                self.cache = IdTags::default();
                ClearCacheResponse {
                    status: GenericStatus::Accepted,
                }
            }),
            "TriggerMessage" => answer(payload, |request: TriggerMessageRequest| {
                TriggerMessageResponse {
                    status: self.trigger(now, request),
                }
            }),
            "ReserveNow" => answer(payload, |request: ReserveNowRequest| ReserveNowResponse {
                status: self.reserve_now(out, now, request),
            }),
            "CancelReservation" => answer(payload, |request: CancelReservationRequest| {
                CancelReservationResponse {
                    status: self.cancel_reservation(out, now, request),
                }
            }),
            "SendLocalList" => answer(payload, |request: SendLocalListRequest| {
                SendLocalListResponse {
                    status: self.send_local_list(out, now, &request),
                }
            }),
            "GetLocalListVersion" => answer(payload, |_: GetLocalListVersionRequest| {
                GetLocalListVersionResponse {
                    list_version: self.local_list.version(),
                }
            }),
            "SetChargingProfile" => answer(payload, |request: SetChargingProfileRequest| {
                SetChargingProfileResponse {
                    status: self.set_charging_profile(out, now, request),
                }
            }),
            "ClearChargingProfile" => answer(payload, |request: ClearChargingProfileRequest| {
                ClearChargingProfileResponse {
                    status: self.clear_charging_profile(out, now, request),
                }
            }),
            "GetCompositeSchedule" => answer(payload, |request: GetCompositeScheduleRequest| {
                self.composite_schedule(now, request)
            }),
            "UpdateFirmware" => answer(payload, |request: UpdateFirmwareRequest| {
                self.update_firmware(now, request);
                UpdateFirmwareResponse {}
            }),
            "GetDiagnostics" => answer(payload, |request: GetDiagnosticsRequest| {
                self.get_diagnostics(out, now, request)
            }),
            "ExtendedTriggerMessage" => {
                answer(payload, |request: ExtendedTriggerMessageRequest| {
                    ExtendedTriggerMessageResponse {
                        status: self.extended_trigger(now, request),
                    }
                })
            }
            "GetLog" => answer(payload, |request: GetLogRequest| {
                self.get_log(out, now, request)
            }),
            "SignedUpdateFirmware" => answer(payload, |request: SignedUpdateFirmwareRequest| {
                SignedUpdateFirmwareResponse {
                    status: self.signed_update_firmware(out, now, request),
                }
            }),
            "InstallCertificate" => answer(payload, |request: InstallCertificateRequest| {
                self.install_certificate(out, now, request)
            }),
            "GetInstalledCertificateIds" => {
                answer(payload, |request: GetInstalledCertificateIdsRequest| {
                    self.installed_certificate_ids(request)
                })
            }
            "DeleteCertificate" => answer(payload, |request: DeleteCertificateRequest| {
                self.delete_certificate(out, now, request)
            }),
            "CertificateSigned" => answer(payload, |request: CertificateSignedRequest| {
                self.certificate_signed(out, now, request)
            }),
            "DataTransfer" => answer(payload, |_: DataTransferRequest| DataTransferResponse {
                status: DataTransferStatus::UnknownVendorId,
                data: None,
            }),
            _ => Err((
                ErrorCode::NotImplemented,
                format!("{action} is not implemented"),
            )),
        };
        let frame = match answered {
            Ok(payload) => Frame::CallResult { id, payload },
            Err((code, description)) => {
                if let ErrorCode::Invalid(_) = code {
                    let kind = EventKind::Link(LinkEvent::InvalidPayload {
                        action: action.to_owned(),
                        reason: description.clone(),
                    });
                    self.emit(out, now, kind);
                }
                Frame::CallError {
                    id,
                    code: code.as_str().into(),
                    description,
                    details: json!({}),
                }
            }
        };
        out.push(Output::Send(frame));
    }

    pub(super) fn configure_locally(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        key: &str,
        value: &str,
    ) {
        let (Ok(name), Ok(text)) = (CiString::new(key), CiString::new(value)) else {
            return self.reject(out, now, None, &format!("{key} refused: too long"));
        };
        let request = ChangeConfigurationRequest {
            key: name,
            value: text,
        };
        if self.configuration.is_read_only(key) {
            return self.configure_read_only(out, now, &request);
        }
        let status = self.change_configuration(out, now, &request).status;
        if !matches!(
            status,
            ConfigurationStatus::Accepted | ConfigurationStatus::RebootRequired
        ) {
            self.reject(out, now, None, &format!("{key} refused ({status:?})"));
        }
    }

    fn configure_read_only(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: &ChangeConfigurationRequest,
    ) {
        let key = request.key.as_str();
        if self.is_connected() {
            return self.reject(
                out,
                now,
                None,
                &format!("{key} refused: read-only while connected"),
            );
        }
        if is_derived(key) {
            return self.reject(
                out,
                now,
                None,
                &format!("{key} refused: derived from the station"),
            );
        }
        if self.configuration.install(request) != ConfigurationStatus::Accepted {
            return self.reject(out, now, None, &format!("{key} refused: not a valid value"));
        }
        let kind = EventKind::ConfigurationChanged {
            key: key.to_owned(),
            value: request.value.as_str().to_owned(),
        };
        self.emit(out, now, kind);
    }

    fn change_configuration(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: &ChangeConfigurationRequest,
    ) -> ChangeConfigurationResponse {
        let status = match self.security_prerequisites_met(request) {
            true => self.configuration.change(request),
            false => ConfigurationStatus::Rejected,
        };
        if status != ConfigurationStatus::Accepted {
            return ChangeConfigurationResponse { status };
        }
        let key = request.key.as_str();
        let secret = key.eq_ignore_ascii_case(AUTHORIZATION_KEY);
        if !secret {
            let kind = EventKind::ConfigurationChanged {
                key: key.to_owned(),
                value: request.value.as_str().to_owned(),
            };
            self.emit(out, now, kind);
        }
        if secret || key.eq_ignore_ascii_case(SECURITY_PROFILE) {
            self.security_event(out, now, RECONFIGURATION_OF_SECURITY_PARAMETERS);
            self.reconnect_requested = true;
        }
        ChangeConfigurationResponse { status }
    }

    fn send_local_list(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: &SendLocalListRequest,
    ) -> UpdateStatus {
        let limit = |key| self.configuration.number(key).unwrap_or(0) as usize;
        let limits = Limits {
            per_update: limit(SEND_LOCAL_LIST_MAX_LENGTH),
            total: limit(LOCAL_AUTH_LIST_MAX_LENGTH),
        };
        let status = self.local_list.update(request, limits);
        if status == UpdateStatus::Accepted {
            let kind = EventKind::LocalListUpdated {
                version: self.local_list.version(),
                entries: self.local_list.len(),
            };
            self.emit(out, now, kind);
        }
        status
    }
}

fn answer<Request: DeserializeOwned + Serialize, Response: Serialize>(
    payload: Value,
    handler: impl FnOnce(Request) -> Response,
) -> Result<Value, (ErrorCode, String)> {
    let request = payload::decode(payload)
        .map_err(|invalid| (ErrorCode::Invalid(invalid.violation), invalid.reason))?;
    Ok(json!(handler(request)))
}

#[cfg(test)]
#[path = "csms_call.test.rs"]
mod tests;
