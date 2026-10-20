use super::ChargePoint;
use super::diagnostics::UPLOAD_MS;
use super::firmware_update::reachable;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::security::log::{
    GetLogRequest, GetLogResponse, LogStatus, LogStatusNotificationRequest, LogType,
    UploadLogStatus,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

pub struct LogUpload {
    request_id: i32,
    until: u64,
}

impl ChargePoint {
    pub(super) fn get_log(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: GetLogRequest,
    ) -> GetLogResponse {
        let kind = match request.log_type {
            LogType::DiagnosticsLog => "diagnostics",
            LogType::SecurityLog => "security",
        };
        let Ok(filename) = CiString::new(&format!("{}-{kind}.log", self.station.id)) else {
            return GetLogResponse {
                status: LogStatus::Rejected,
                filename: None,
            };
        };
        let status = match self.log_upload {
            Some(_) => LogStatus::AcceptedCanceled,
            None => LogStatus::Accepted,
        };
        let request_id = request.request_id;
        if reachable(request.log.remote_location.as_str()) {
            let until = now + UPLOAD_MS;
            self.log_upload = Some(LogUpload { request_id, until });
            self.log_status(out, now, UploadLogStatus::Uploading, Some(request_id));
        } else {
            self.log_upload = None;
            self.log_status(out, now, UploadLogStatus::UploadFailure, Some(request_id));
        }
        GetLogResponse {
            status,
            filename: Some(filename),
        }
    }

    pub(super) fn advance_log_upload(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(upload) = &self.log_upload else {
            return;
        };
        if now < upload.until {
            return;
        }
        let request_id = upload.request_id;
        self.log_upload = None;
        self.log_status(out, now, UploadLogStatus::Uploaded, Some(request_id));
    }

    pub(super) fn log_upload_deadline(&self) -> Option<u64> {
        self.log_upload.as_ref().map(|upload| upload.until)
    }

    pub(super) fn report_log_status(&mut self) {
        match &self.log_upload {
            Some(upload) => {
                let request_id = Some(upload.request_id);
                self.push_log_status(UploadLogStatus::Uploading, request_id);
            }
            None => self.push_log_status(UploadLogStatus::Idle, None),
        }
    }

    fn log_status(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        status: UploadLogStatus,
        request_id: Option<i32>,
    ) {
        self.push_log_status(status, request_id);
        self.emit(out, now, EventKind::LogStatus { status, request_id });
    }

    fn push_log_status(&mut self, status: UploadLogStatus, request_id: Option<i32>) {
        let request = LogStatusNotificationRequest { status, request_id };
        self.outbox.push("LogStatusNotification", 0, json!(request));
    }
}

#[cfg(test)]
#[path = "log_upload.test.rs"]
mod tests;
