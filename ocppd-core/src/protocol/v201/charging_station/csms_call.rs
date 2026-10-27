use super::ChargingStation;
use super::device_model::table::{ITEMS_PER_GET, ITEMS_PER_REPORT};
use super::diagnostics::monitoring::ITEMS_PER_MONITORING;
use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::authorization::ClearCacheRequest;
use crate::protocol::v201::messages::availability::ChangeAvailabilityRequest;
use crate::protocol::v201::messages::custom_data::without_custom_data;
use crate::protocol::v201::messages::data_transfer::{
    DataTransferRequest, DataTransferResponse, DataTransferStatus,
};
use crate::protocol::v201::messages::diagnostics::customer_information::CustomerInformationRequest;
use crate::protocol::v201::messages::diagnostics::log::GetLogRequest;
use crate::protocol::v201::messages::diagnostics::monitoring::{
    ClearVariableMonitoringRequest, SetMonitoringBaseRequest, SetMonitoringLevelRequest,
    SetVariableMonitoringRequest,
};
use crate::protocol::v201::messages::diagnostics::monitoring_report::GetMonitoringReportRequest;
use crate::protocol::v201::messages::display::{
    ClearDisplayMessageRequest, GetDisplayMessagesRequest, SetDisplayMessageRequest,
};
use crate::protocol::v201::messages::firmware::UpdateFirmwareRequest;
use crate::protocol::v201::messages::local_auth::{
    GetLocalListVersionRequest, SendLocalListRequest,
};
use crate::protocol::v201::messages::provisioning::network_profile::SetNetworkProfileRequest;
use crate::protocol::v201::messages::provisioning::report::{
    GetBaseReportRequest, GetReportRequest,
};
use crate::protocol::v201::messages::provisioning::reset::ResetRequest;
use crate::protocol::v201::messages::provisioning::variables::{
    GetVariablesRequest, SetVariablesRequest,
};
use crate::protocol::v201::messages::remote_control::{
    RequestStartTransactionRequest, RequestStopTransactionRequest, TriggerMessageRequest,
    TriggerMessageResponse, UnlockConnectorRequest,
};
use crate::protocol::v201::messages::reservation::{CancelReservationRequest, ReserveNowRequest};
use crate::protocol::v201::messages::security::certificates::{
    CertificateSignedRequest, DeleteCertificateRequest, GetInstalledCertificateIdsRequest,
    InstallCertificateRequest,
};
use crate::protocol::v201::messages::smart_charging::SetChargingProfileRequest;
use crate::protocol::v201::messages::smart_charging::clear::ClearChargingProfileRequest;
use crate::protocol::v201::messages::smart_charging::composite::GetCompositeScheduleRequest;
use crate::protocol::v201::messages::smart_charging::report::GetChargingProfilesRequest;
use crate::protocol::v201::messages::tariff_cost::CostUpdatedRequest;
use crate::protocol::v201::messages::transaction::GetTransactionStatusRequest;
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
            "SetVariableMonitoring" => answer(payload, |request: SetVariableMonitoringRequest| {
                if request.set_monitoring_data.len() > ITEMS_PER_MONITORING {
                    return Err((
                        Violation::Occurrence,
                        format!("more than {ITEMS_PER_MONITORING} monitors"),
                    ));
                }
                Ok(self.set_variable_monitoring(now, request))
            }),
            "ClearVariableMonitoring" => {
                answer(payload, |request: ClearVariableMonitoringRequest| {
                    if request.id.len() > ITEMS_PER_MONITORING {
                        return Err((
                            Violation::Occurrence,
                            format!("more than {ITEMS_PER_MONITORING} ids"),
                        ));
                    }
                    Ok(self.clear_variable_monitoring(request))
                })
            }
            "SetMonitoringBase" => answer(payload, |request: SetMonitoringBaseRequest| {
                Ok(self.set_monitoring_base(request))
            }),
            "GetMonitoringReport" => answer(payload, |request: GetMonitoringReportRequest| {
                let asked = request.component_variable.as_ref().map_or(0, Vec::len);
                if asked > ITEMS_PER_REPORT {
                    return Err((
                        Violation::Occurrence,
                        format!("more than {ITEMS_PER_REPORT} component variables"),
                    ));
                }
                Ok(self.get_monitoring_report(now, request))
            }),
            "SetMonitoringLevel" => answer(payload, |request: SetMonitoringLevelRequest| {
                Ok(self.set_monitoring_level(request))
            }),
            "GetReport" => answer(payload, |request: GetReportRequest| {
                let asked = request.component_variable.as_ref().map_or(0, Vec::len);
                if asked > ITEMS_PER_REPORT {
                    return Err((
                        Violation::Occurrence,
                        format!("more than {ITEMS_PER_REPORT} component variables"),
                    ));
                }
                Ok(self.get_report(now, request))
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
            "GetChargingProfiles" => answer(payload, |request: GetChargingProfilesRequest| {
                Ok(self.get_charging_profiles(request))
            }),
            "ClearChargingProfile" => answer(payload, |request: ClearChargingProfileRequest| {
                Ok(self.clear_charging_profile(out, now, request))
            }),
            "GetCompositeSchedule" => answer(payload, |request: GetCompositeScheduleRequest| {
                Ok(self.composite_schedule(now, request))
            }),
            "ReserveNow" => answer(payload, |request: ReserveNowRequest| {
                Ok(self.reserve_now(out, now, request))
            }),
            "CancelReservation" => answer(payload, |request: CancelReservationRequest| {
                Ok(self.cancel_reservation(out, now, request))
            }),
            "SetDisplayMessage" => answer(payload, |request: SetDisplayMessageRequest| {
                Ok(self.set_display_message(out, now, request))
            }),
            "GetDisplayMessages" => answer(payload, |request: GetDisplayMessagesRequest| {
                Ok(self.get_display_messages(request))
            }),
            "ClearDisplayMessage" => answer(payload, |request: ClearDisplayMessageRequest| {
                Ok(self.clear_display_message(out, now, request))
            }),
            "CostUpdated" => answer(payload, |request: CostUpdatedRequest| {
                Ok(self.cost_updated(out, now, request))
            }),
            "DataTransfer" => answer(payload, |_: DataTransferRequest| {
                Ok(DataTransferResponse {
                    status: DataTransferStatus::UnknownVendorId,
                    status_info: None,
                    data: None,
                })
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

#[cfg(test)]
#[path = "csms_call.test.rs"]
mod tests;
