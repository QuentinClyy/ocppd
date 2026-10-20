use super::ChargePoint;
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
            "ClearCache" => answer(payload, |_: ClearCacheRequest| ClearCacheResponse {
                status: GenericStatus::Accepted,
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
        if status == ConfigurationStatus::Accepted {
            let kind = EventKind::ConfigurationChanged {
                key: request.key.as_str().to_owned(),
                value: request.value.as_str().to_owned(),
            };
            self.emit(out, now, kind);
        }
        ChangeConfigurationResponse { status }
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
