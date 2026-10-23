use super::ChargingStation;
use super::security::{
    FIRMWARE_UPDATED, INVALID_FIRMWARE_SIGNATURE, INVALID_FIRMWARE_SIGNING_CERTIFICATE,
};
use crate::api::io::Output;
use crate::protocol::shared::file_transfer::{DOWNLOAD_MS, TransferStage, reachable};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::firmware::{
    FirmwareStatus, FirmwareStatusNotificationRequest, UpdateFirmwareRequest,
    UpdateFirmwareResponse, UpdateFirmwareStatus,
};
use crate::protocol::v201::messages::provisioning::boot::BootReason;
use crate::wire::ci_string::CiString;
use serde_json::json;

const INSTALL_MS: u64 = 10_000;

pub struct Firmware {
    update: Option<FirmwareUpdate>,
    last_sent: (FirmwareStatus, Option<i32>),
    rebooting_for: Option<i32>,
}

impl Default for Firmware {
    fn default() -> Self {
        Firmware {
            update: None,
            last_sent: (FirmwareStatus::Idle, None),
            rebooting_for: None,
        }
    }
}

struct FirmwareUpdate {
    request_id: i32,
    location: String,
    has_signature: bool,
    install_at: u64,
    stage: Stage,
}

#[derive(Clone, Copy)]
enum Stage {
    Waiting { until: u64 },
    Downloading { until: u64 },
    Downloaded,
    Installing { until: u64 },
}

impl ChargingStation {
    pub(super) fn update_firmware(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: UpdateFirmwareRequest,
    ) -> UpdateFirmwareResponse {
        let firmware = request.firmware;
        let Some(signer) = firmware.signing_certificate.as_ref() else {
            return response(UpdateFirmwareStatus::Rejected);
        };
        if !self.trusted_firmware_signer(signer.as_str(), now) {
            self.security_event(out, now, INVALID_FIRMWARE_SIGNING_CERTIFICATE);
            return response(UpdateFirmwareStatus::InvalidCertificate);
        }
        let status = match self.firmware.update {
            Some(_) => UpdateFirmwareStatus::AcceptedCanceled,
            None => UpdateFirmwareStatus::Accepted,
        };
        let until = firmware.retrieve_date_time.unix_ms.max(now);
        let request_id = request.request_id;
        self.firmware.update = Some(FirmwareUpdate {
            request_id,
            location: firmware.location.as_str().to_owned(),
            has_signature: firmware.signature.is_some(),
            install_at: firmware.install_date_time.map_or(0, |at| at.unix_ms),
            stage: Stage::Waiting { until },
        });
        if until > now {
            self.firmware_status(out, now, FirmwareStatus::DownloadScheduled, request_id);
        }
        response(status)
    }

    pub(super) fn advance_firmware(&mut self, out: &mut Vec<Output>, now: u64) {
        while let Some(update) = &self.firmware.update {
            let request_id = update.request_id;
            let next = match update.stage {
                Stage::Waiting { until } if now >= until => {
                    if !reachable(&update.location) {
                        let failed = FirmwareStatus::DownloadFailed;
                        return self.end_update(out, now, failed, request_id);
                    }
                    self.firmware_status(out, now, FirmwareStatus::Downloading, request_id);
                    Stage::Downloading {
                        until: until + DOWNLOAD_MS,
                    }
                }
                Stage::Downloading { until } if now >= until => {
                    if self.failing.take(TransferStage::Download) {
                        let failed = FirmwareStatus::DownloadFailed;
                        return self.end_update(out, now, failed, request_id);
                    }
                    let signed = update.has_signature;
                    let scheduled = update.install_at > now;
                    self.firmware_status(out, now, FirmwareStatus::Downloaded, request_id);
                    if !signed || self.failing.take(TransferStage::Signature) {
                        let invalid = FirmwareStatus::InvalidSignature;
                        self.end_update(out, now, invalid, request_id);
                        return self.security_event(out, now, INVALID_FIRMWARE_SIGNATURE);
                    }
                    let verified = FirmwareStatus::SignatureVerified;
                    self.firmware_status(out, now, verified, request_id);
                    if scheduled {
                        let scheduled = FirmwareStatus::InstallScheduled;
                        self.firmware_status(out, now, scheduled, request_id);
                    }
                    Stage::Downloaded
                }
                Stage::Downloaded if now >= update.install_at && self.idle() => {
                    self.firmware_status(out, now, FirmwareStatus::Installing, request_id);
                    Stage::Installing {
                        until: now + INSTALL_MS,
                    }
                }
                Stage::Installing { until } if now >= until => {
                    return self.install_firmware(out, now);
                }
                _ => return,
            };
            if let Some(update) = &mut self.firmware.update {
                update.stage = next;
            }
        }
    }

    pub(super) fn firmware_deadline(&self) -> Option<u64> {
        let update = self.firmware.update.as_ref()?;
        match update.stage {
            Stage::Waiting { until }
            | Stage::Downloading { until }
            | Stage::Installing { until } => Some(until),
            Stage::Downloaded => Some(update.install_at).filter(|&at| at > 0),
        }
    }

    pub(super) fn installed_after_reboot(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(request_id) = self.firmware.rebooting_for.take() else {
            return;
        };
        self.firmware_status(out, now, FirmwareStatus::Installed, request_id);
        self.security_event(out, now, FIRMWARE_UPDATED);
    }

    pub(super) fn report_firmware_status(&mut self) {
        let (status, request_id) = match self.firmware.last_sent {
            (FirmwareStatus::Installed, _) => (FirmwareStatus::Idle, None),
            last_sent => last_sent,
        };
        let request = FirmwareStatusNotificationRequest { status, request_id };
        self.outbox
            .push("FirmwareStatusNotification", 0, json!(request));
    }

    fn install_firmware(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(update) = self.firmware.update.take() else {
            return;
        };
        let version = update.location.rsplit('/').next().unwrap_or_default();
        let boot_version = Some(version)
            .filter(|version| !version.is_empty())
            .and_then(|version| CiString::new(version).ok());
        let boot_version = boot_version.filter(|_| !self.failing.take(TransferStage::Install));
        let Some(boot_version) = boot_version else {
            let failed = FirmwareStatus::InstallationFailed;
            return self.firmware_status(out, now, failed, update.request_id);
        };
        let rebooting = FirmwareStatus::InstallRebooting;
        self.firmware_status(out, now, rebooting, update.request_id);
        self.firmware.rebooting_for = Some(update.request_id);
        self.boot_request.charging_station.firmware_version = Some(boot_version);
        self.station.firmware_version = version.to_owned();
        self.reboot = Some(BootReason::FirmwareUpdate);
    }

    fn end_update(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        status: FirmwareStatus,
        request_id: i32,
    ) {
        self.firmware.update = None;
        self.firmware_status(out, now, status, request_id);
    }

    fn firmware_status(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        status: FirmwareStatus,
        request_id: i32,
    ) {
        self.firmware.last_sent = (status, Some(request_id));
        let request = FirmwareStatusNotificationRequest {
            status,
            request_id: Some(request_id),
        };
        self.outbox
            .push("FirmwareStatusNotification", 0, json!(request));
        self.emit(out, now, EventKind::FirmwareStatus { status, request_id });
    }

    fn idle(&self) -> bool {
        self.sessions
            .iter()
            .all(|session| session.transaction.is_none())
    }
}

fn response(status: UpdateFirmwareStatus) -> UpdateFirmwareResponse {
    UpdateFirmwareResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "firmware.test.rs"]
mod tests;
