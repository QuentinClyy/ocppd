use crate::api::io::Output;
use crate::protocol::shared::file_transfer::{TransferStage, UPLOAD_MS, reachable};
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::firmware::{
    DiagnosticsStatus, DiagnosticsStatusNotificationRequest, GetDiagnosticsRequest,
    GetDiagnosticsResponse,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

impl ChargePoint {
    pub(crate) fn get_diagnostics(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: GetDiagnosticsRequest,
    ) -> GetDiagnosticsResponse {
        let file_name = format!("{}-diagnostics.log", self.station.id);
        let Ok(file_name) = CiString::new(&file_name) else {
            return GetDiagnosticsResponse { file_name: None };
        };
        if reachable(request.location.as_str()) {
            self.uploading_until = Some(now + UPLOAD_MS);
            self.diagnostics_status(out, now, DiagnosticsStatus::Uploading);
        } else {
            self.uploading_until = None;
            self.diagnostics_status(out, now, DiagnosticsStatus::UploadFailed);
        }
        GetDiagnosticsResponse {
            file_name: Some(file_name),
        }
    }

    pub(crate) fn advance_diagnostics(&mut self, out: &mut Vec<Output>, now: u64) {
        if self.uploading_until.is_some_and(|until| now >= until) {
            self.uploading_until = None;
            let status = match self.failing.take(TransferStage::Upload) {
                true => DiagnosticsStatus::UploadFailed,
                false => DiagnosticsStatus::Uploaded,
            };
            self.diagnostics_status(out, now, status);
        }
    }

    pub(crate) fn current_diagnostics_status(&self) -> DiagnosticsStatus {
        match self.uploading_until {
            Some(_) => DiagnosticsStatus::Uploading,
            None => DiagnosticsStatus::Idle,
        }
    }

    fn diagnostics_status(&mut self, out: &mut Vec<Output>, now: u64, status: DiagnosticsStatus) {
        let request = DiagnosticsStatusNotificationRequest { status };
        self.outbox
            .push("DiagnosticsStatusNotification", 0, json!(request));
        self.emit(out, now, EventKind::DiagnosticsStatus { status });
    }
}

#[cfg(test)]
#[path = "diagnostics.test.rs"]
mod tests;
