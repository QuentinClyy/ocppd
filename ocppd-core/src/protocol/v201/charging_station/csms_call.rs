use super::ChargingStation;
use super::device_model::table::ITEMS_PER_GET;
use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::authorization::ClearCacheRequest;
use crate::protocol::v201::messages::availability::ChangeAvailabilityRequest;
use crate::protocol::v201::messages::custom_data::without_custom_data;
use crate::protocol::v201::messages::diagnostics::customer_information::CustomerInformationRequest;
use crate::protocol::v201::messages::diagnostics::log::GetLogRequest;
use crate::protocol::v201::messages::firmware::UpdateFirmwareRequest;
use crate::protocol::v201::messages::local_auth::{
    GetLocalListVersionRequest, SendLocalListRequest,
};
use crate::protocol::v201::messages::provisioning::network_profile::SetNetworkProfileRequest;
use crate::protocol::v201::messages::provisioning::report::GetBaseReportRequest;
use crate::protocol::v201::messages::provisioning::reset::ResetRequest;
use crate::protocol::v201::messages::provisioning::variables::{
    GetVariablesRequest, SetVariablesRequest,
};
use crate::protocol::v201::messages::remote_control::{
    RequestStartTransactionRequest, RequestStopTransactionRequest, TriggerMessageRequest,
    TriggerMessageResponse, UnlockConnectorRequest,
};
use crate::protocol::v201::messages::security::certificates::{
    CertificateSignedRequest, DeleteCertificateRequest, GetInstalledCertificateIdsRequest,
    InstallCertificateRequest,
};
use crate::protocol::v201::messages::smart_charging::SetChargingProfileRequest;
use crate::protocol::v201::messages::transactions::GetTransactionStatusRequest;
use crate::wire::frame::Frame;
use crate::wire::payload::{self, Violation};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

const INSTRUCTING: [&str; 3] = ["TriggerMessage", "GetBaseReport", "GetReport"];

enum ErrorCode {
    NotImplemented,
    SecurityError,
    Invalid(Violation),
}

impl ErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::NotImplemented => "NotImplemented",
            ErrorCode::SecurityError => "SecurityError",
            ErrorCode::Invalid(Violation::Formation) => "FormatViolation",
            ErrorCode::Invalid(Violation::Occurrence) => "OccurrenceConstraintViolation",
            ErrorCode::Invalid(Violation::Type) => "TypeConstraintViolation",
            ErrorCode::Invalid(Violation::Property) => "PropertyConstraintViolation",
        }
    }
}

impl ChargingStation {
    pub(super) fn csms_call(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        id: String,
        action: &str,
        payload: Value,
    ) {
        self.outbox.set_instructing(INSTRUCTING.contains(&action));
        let answered = match action {
            _ if self.is_rejected() => Err((
                ErrorCode::SecurityError,
                "the CSMS rejected the boot".to_owned(),
            )),
            "GetVariables" => answer(payload, |request: GetVariablesRequest| {
                if request.get_variable_data.len() > ITEMS_PER_GET {
                    return Err((
                        Violation::Occurrence,
                        format!("more than {ITEMS_PER_GET} variables"),
                    ));
                }
                Ok(self.get_variables(now, request))
            }),
            "SetVariables" => answer(payload, |request: SetVariablesRequest| {
                Ok(self.set_variables(out, now, request))
            }),
            "GetBaseReport" => answer(payload, |request: GetBaseReportRequest| {
                Ok(self.get_base_report(now, request))
            }),
            "GetTransactionStatus" => answer(payload, |request: GetTransactionStatusRequest| {
                Ok(self.transaction_status(request))
            }),
            "RequestStartTransaction" => {
                answer(payload, |request: RequestStartTransactionRequest| {
                    Ok(self.request_start(out, now, request))
                })
            }
            "RequestStopTransaction" => {
                answer(payload, |request: RequestStopTransactionRequest| {
                    Ok(self.request_stop(out, now, request))
                })
            }
            "UnlockConnector" => answer(payload, |request: UnlockConnectorRequest| {
                Ok(self.unlock_connector(out, now, request))
            }),
            "TriggerMessage" => answer(payload, |request: TriggerMessageRequest| {
                let status = self.trigger_message(out, now, request);
                Ok(TriggerMessageResponse {
                    status,
                    status_info: None,
                })
            }),
            "ChangeAvailability" => answer(payload, |request: ChangeAvailabilityRequest| {
                Ok(self.change_availability(out, now, request))
            }),
            "ClearCache" => answer(payload, |_: ClearCacheRequest| Ok(self.clear_cache())),
            "SendLocalList" => answer(payload, |request: SendLocalListRequest| {
                Ok(self.send_local_list(out, now, &request))
            }),
            "GetLocalListVersion" => answer(payload, |_: GetLocalListVersionRequest| {
                Ok(self.local_list_version())
            }),
            "InstallCertificate" => answer(payload, |request: InstallCertificateRequest| {
                Ok(self.install_certificate(out, now, request))
            }),
            "GetInstalledCertificateIds" => {
                answer(payload, |request: GetInstalledCertificateIdsRequest| {
                    Ok(self.installed_certificate_ids(request))
                })
            }
            "CertificateSigned" => answer(payload, |request: CertificateSignedRequest| {
                Ok(self.certificate_signed(out, now, request))
            }),
            "DeleteCertificate" => answer(payload, |request: DeleteCertificateRequest| {
                Ok(self.delete_certificate(out, now, request))
            }),
            "UpdateFirmware" => answer(payload, |request: UpdateFirmwareRequest| {
                Ok(self.update_firmware(out, now, request))
            }),
            "GetLog" => answer(payload, |request: GetLogRequest| {
                Ok(self.get_log(out, now, request))
            }),
            "CustomerInformation" => answer(payload, |request: CustomerInformationRequest| {
                Ok(self.customer_information(out, now, request))
            }),
            "SetNetworkProfile" => answer(payload, |request: SetNetworkProfileRequest| {
                Ok(self.set_network_profile(out, now, request))
            }),
            "SetChargingProfile" => answer(payload, |request: SetChargingProfileRequest| {
                Ok(self.set_charging_profile(out, now, request))
            }),
            "Reset" => answer(payload, |request: ResetRequest| {
                Ok(self.reset(out, now, request))
            }),
            _ => Err((
                ErrorCode::NotImplemented,
                format!("{action} is not implemented"),
            )),
        };
        self.outbox.set_instructing(false);
        let frame = match answered {
            Ok(payload) => Frame::CallResult { id, payload },
            Err((code, description)) => {
                if let ErrorCode::Invalid(_) = code {
                    let action = action.to_owned();
                    let reason = description.clone();
                    let kind = LinkEvent::InvalidPayload { action, reason };
                    self.emit(out, now, EventKind::Link(kind));
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
}

fn answer<Request: DeserializeOwned + Serialize, Response: Serialize>(
    payload: Value,
    handler: impl FnOnce(Request) -> Result<Response, (Violation, String)>,
) -> Result<Value, (ErrorCode, String)> {
    let request = payload::decode(without_custom_data(payload))
        .map_err(|invalid| (ErrorCode::Invalid(invalid.violation), invalid.reason))?;
    let response =
        handler(request).map_err(|(violation, reason)| (ErrorCode::Invalid(violation), reason))?;
    Ok(json!(response))
}
