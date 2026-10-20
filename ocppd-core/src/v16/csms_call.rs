use super::charge_point::ChargePoint;
use super::messages::configuration::{
    ChangeConfigurationRequest, ChangeConfigurationResponse, ConfigurationStatus,
    GetConfigurationRequest,
};
use crate::event::{EventKind, Output};
use crate::frame::Frame;
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
