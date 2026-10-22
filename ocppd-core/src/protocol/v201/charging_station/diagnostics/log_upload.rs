use crate::api::io::Output;
use crate::file_transfer::{UPLOAD_MS, reachable};
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::diagnostics::log::{
    GetLogRequest, GetLogResponse, LogStatus, LogStatusNotificationRequest, LogType,
    UploadLogStatus,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

pub struct LogUpload {
    request_id: i32,
    until: u64,
}

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn get_log(
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
                status_info: None,
                filename: None,
            };
        };
        let status = match self.log_upload.take() {
            Some(canceled) => {
                let canceled_status = UploadLogStatus::AcceptedCanceled;
                self.log_status(out, now, canceled_status, canceled.request_id);
                LogStatus::AcceptedCanceled
            }
            None => LogStatus::Accepted,
        };
        let request_id = request.request_id;
        if reachable(request.log.remote_location.as_str()) {
            let until = now + UPLOAD_MS;
            self.log_upload = Some(LogUpload { request_id, until });
            self.log_status(out, now, UploadLogStatus::Uploading, request_id);
        } else {
            self.log_status(out, now, UploadLogStatus::UploadFailure, request_id);
        }
        GetLogResponse {
            status,
            status_info: None,
            filename: Some(filename),
        }
    }

    pub(in crate::protocol::v201::charging_station) fn advance_log_upload(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
    ) {
        let Some(upload) = self.log_upload.take_if(|upload| now >= upload.until) else {
            return;
        };
        self.log_status(out, now, UploadLogStatus::Uploaded, upload.request_id);
    }

    pub(in crate::protocol::v201::charging_station) fn log_upload_deadline(&self) -> Option<u64> {
        self.log_upload.as_ref().map(|upload| upload.until)
    }

    pub(in crate::protocol::v201::charging_station) fn report_log_status(&mut self) {
        let request = match &self.log_upload {
            Some(upload) => LogStatusNotificationRequest {
                status: UploadLogStatus::Uploading,
                request_id: Some(upload.request_id),
            },
            None => LogStatusNotificationRequest {
                status: UploadLogStatus::Idle,
                request_id: None,
            },
        };
        self.outbox.push("LogStatusNotification", 0, json!(request));
    }

    fn log_status(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        status: UploadLogStatus,
        request_id: i32,
    ) {
        let request = LogStatusNotificationRequest {
            status,
            request_id: Some(request_id),
        };
        self.outbox.push("LogStatusNotification", 0, json!(request));
        self.emit(out, now, EventKind::LogStatus { status, request_id });
    }
}

#[cfg(test)]
#[path = "log_upload.test.rs"]
mod tests;
