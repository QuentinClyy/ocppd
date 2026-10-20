use super::ChargePoint;
use super::configuration::{
    AUTHORIZATION_KEY, LOCAL_AUTH_LIST_MAX_LENGTH, SECURITY_PROFILE, SEND_LOCAL_LIST_MAX_LENGTH,
};
use super::id_tags::IdTags;
use super::local_list::Limits;
use super::security_event::RECONFIGURATION_OF_SECURITY_PARAMETERS;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::core::configuration::{
    ChangeConfigurationRequest, ChangeConfigurationResponse, ConfigurationStatus,
    GetConfigurationRequest,
};
use crate::v16::messages::core::data_transfer::{
    DataTransferRequest, DataTransferResponse, DataTransferStatus,
};
use crate::v16::messages::core::remote_control::{
    ChangeAvailabilityRequest, ChangeAvailabilityResponse, ClearCacheRequest, ClearCacheResponse,
    RemoteStartTransactionRequest, RemoteStartTransactionResponse, RemoteStopTransactionRequest,
    RemoteStopTransactionResponse, ResetRequest, ResetResponse, UnlockConnectorRequest,
    UnlockConnectorResponse,
};
use crate::v16::messages::datatypes::generic_status::GenericStatus;
use crate::v16::messages::firmware::{
    GetDiagnosticsRequest, UpdateFirmwareRequest, UpdateFirmwareResponse,
};
use crate::v16::messages::local_auth::{
    GetLocalListVersionRequest, GetLocalListVersionResponse, SendLocalListRequest,
    SendLocalListResponse, UpdateStatus,
};
use crate::v16::messages::remote_trigger::{
    ExtendedTriggerMessageRequest, ExtendedTriggerMessageResponse, TriggerMessageRequest,
    TriggerMessageResponse,
};
use crate::v16::messages::reservation::{
    CancelReservationRequest, CancelReservationResponse, ReserveNowRequest, ReserveNowResponse,
};
use crate::v16::messages::smart_charging::{
    ClearChargingProfileRequest, ClearChargingProfileResponse, GetCompositeScheduleRequest,
    SetChargingProfileRequest, SetChargingProfileResponse,
};
use crate::wire::frame::Frame;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

enum ErrorCode {
    NotImplemented,
    FormationViolation,
}

impl ErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::NotImplemented => "NotImplemented",
            ErrorCode::FormationViolation => "FormationViolation",
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
                if let ErrorCode::FormationViolation = code {
                    let kind = EventKind::InvalidPayload {
                        action: action.to_owned(),
                        reason: description.clone(),
                    };
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

    fn change_configuration(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: &ChangeConfigurationRequest,
    ) -> ChangeConfigurationResponse {
        let status = self.configuration.change(request);
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

fn answer<Request: DeserializeOwned, Response: Serialize>(
    payload: Value,
    handler: impl FnOnce(Request) -> Response,
) -> Result<Value, (ErrorCode, String)> {
    let request = serde_json::from_value(payload)
        .map_err(|err| (ErrorCode::FormationViolation, err.to_string()))?;
    Ok(json!(handler(request)))
}

#[cfg(test)]
#[path = "csms_call.test.rs"]
mod tests;
