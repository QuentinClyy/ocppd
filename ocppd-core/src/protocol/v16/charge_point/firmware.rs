use crate::api::io::Output;
use crate::protocol::shared::file_transfer::{DOWNLOAD_MS, TransferStage, reachable};
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::security::security_event::{
    FIRMWARE_UPDATED, INVALID_FIRMWARE_SIGNATURE, INVALID_FIRMWARE_SIGNING_CERTIFICATE,
};
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::firmware::{
    FirmwareStatus, FirmwareStatusNotificationRequest, UpdateFirmwareRequest,
};
use crate::protocol::v16::messages::remote_control::ResetType;
use crate::protocol::v16::messages::security::signed_firmware::{
    SignedFirmwareStatus, SignedFirmwareStatusNotificationRequest, SignedUpdateFirmwareRequest,
    UpdateFirmwareStatus,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

const INSTALL_MS: u64 = 10_000;

pub struct FirmwareUpdate {
    location: String,
    signed: Option<Signed>,
    stage: Stage,
}

struct Signed {
    request_id: i32,
    has_signature: bool,
    install_at: u64,
}

#[derive(Clone, Copy)]
enum Stage {
    Waiting { until: u64 },
    Downloading { until: u64 },
    Downloaded,
    Installing { until: u64 },
}

impl ChargePoint {
    pub(crate) fn update_firmware(&mut self, now: u64, request: UpdateFirmwareRequest) {
        let until = request.retrieve_date.unix_ms.max(now);
        self.firmware = Some(FirmwareUpdate {
            location: request.location.into(),
            signed: None,
            stage: Stage::Waiting { until },
        });
    }

    pub(crate) fn signed_update_firmware(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: SignedUpdateFirmwareRequest,
    ) -> UpdateFirmwareStatus {
        let firmware = request.firmware;
        if firmware.signing_certificate.as_str().is_empty() {
            self.security_event(out, now, INVALID_FIRMWARE_SIGNING_CERTIFICATE);
            return UpdateFirmwareStatus::InvalidCertificate;
        }
        let status = match self.firmware {
            Some(_) => UpdateFirmwareStatus::AcceptedCanceled,
            None => UpdateFirmwareStatus::Accepted,
        };
        let until = firmware.retrieve_date_time.unix_ms.max(now);
        let signed = Signed {
            request_id: request.request_id,
            has_signature: !firmware.signature.as_str().is_empty(),
            install_at: firmware.install_date_time.map_or(0, |at| at.unix_ms),
        };
        self.firmware = Some(FirmwareUpdate {
            location: firmware.location.as_str().to_owned(),
            signed: Some(signed),
            stage: Stage::Waiting { until },
        });
        if until > now {
            self.firmware_status(out, now, SignedFirmwareStatus::DownloadScheduled);
        }
        status
    }

    pub(crate) fn advance_firmware(&mut self, out: &mut Vec<Output>, now: u64) {
        while let Some(update) = &self.firmware {
            let next = match update.stage {
                Stage::Waiting { until } if now >= until => {
                    if !reachable(&update.location) {
                        self.firmware_status(out, now, SignedFirmwareStatus::DownloadFailed);
                        self.firmware = None;
                        return;
                    }
                    self.firmware_status(out, now, SignedFirmwareStatus::Downloading);
                    Stage::Downloading {
                        until: until + DOWNLOAD_MS,
                    }
                }
                Stage::Downloading { until } if now >= until => {
                    if self.failing.take(TransferStage::Download) {
                        self.firmware_status(out, now, SignedFirmwareStatus::DownloadFailed);
                        self.firmware = None;
                        return;
                    }
                    self.firmware_status(out, now, SignedFirmwareStatus::Downloaded);
                    if !self.verify_signature(out, now) {
                        self.firmware = None;
                        return;
                    }
                    Stage::Downloaded
                }
                Stage::Downloaded if now >= self.install_at() && self.no_transaction() => {
                    self.firmware_status(out, now, SignedFirmwareStatus::Installing);
                    Stage::Installing {
                        until: now + INSTALL_MS,
                    }
                }
                Stage::Installing { until } if now >= until => {
                    return self.install_firmware(out, now);
                }
                _ => return,
            };
            if let Some(update) = &mut self.firmware {
                update.stage = next;
            }
        }
    }

    pub(crate) fn firmware_deadline(&self) -> Option<u64> {
        match self.firmware.as_ref()?.stage {
            Stage::Waiting { until }
            | Stage::Downloading { until }
            | Stage::Installing { until } => Some(until),
            Stage::Downloaded => Some(self.install_at()).filter(|&at| at > 0),
        }
    }

    pub(crate) fn installed_after_reboot(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(request_id) = self.installed_request.take() else {
            return;
        };
        let status = SignedFirmwareStatus::Installed;
        let request_id = Some(request_id);
        self.push_signed_status(status, request_id);
        self.emit(out, now, EventKind::FirmwareStatus { status, request_id });
        self.security_event(out, now, FIRMWARE_UPDATED);
    }

    pub(crate) fn report_firmware_status(&mut self) {
        let status = plain(self.current_firmware_status()).unwrap_or(FirmwareStatus::Idle);
        let request = FirmwareStatusNotificationRequest { status };
        self.outbox
            .push("FirmwareStatusNotification", 0, json!(request));
    }

    pub(crate) fn report_signed_firmware_status(&mut self) {
        let status = self.current_firmware_status();
        self.push_signed_status(status, self.signed_request_id());
    }

    fn current_firmware_status(&self) -> SignedFirmwareStatus {
        let Some(update) = &self.firmware else {
            return SignedFirmwareStatus::Idle;
        };
        match (update.stage, &update.signed) {
            (Stage::Waiting { .. }, Some(_)) => SignedFirmwareStatus::DownloadScheduled,
            (Stage::Waiting { .. }, None) => SignedFirmwareStatus::Idle,
            (Stage::Downloading { .. }, _) => SignedFirmwareStatus::Downloading,
            (Stage::Downloaded, _) => SignedFirmwareStatus::Downloaded,
            (Stage::Installing { .. }, _) => SignedFirmwareStatus::Installing,
        }
    }

    fn verify_signature(&mut self, out: &mut Vec<Output>, now: u64) -> bool {
        let Some(signed) = self
            .firmware
            .as_ref()
            .and_then(|update| update.signed.as_ref())
        else {
            return true;
        };
        if !signed.has_signature || self.failing.take(TransferStage::Signature) {
            self.firmware_status(out, now, SignedFirmwareStatus::InvalidSignature);
            self.security_event(out, now, INVALID_FIRMWARE_SIGNATURE);
            return false;
        }
        let scheduled = signed.install_at > now;
        self.firmware_status(out, now, SignedFirmwareStatus::SignatureVerified);
        if scheduled {
            self.firmware_status(out, now, SignedFirmwareStatus::InstallScheduled);
        }
        true
    }

    fn install_firmware(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(update) = &self.firmware else {
            return;
        };
        let version = update
            .location
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .to_owned();
        let boot_version = Some(&version)
            .filter(|version| !version.is_empty())
            .and_then(|version| CiString::new(version).ok());
        let boot_version = boot_version.filter(|_| !self.failing.take(TransferStage::Install));
        let Some(boot_version) = boot_version else {
            self.firmware_status(out, now, SignedFirmwareStatus::InstallationFailed);
            self.firmware = None;
            return;
        };
        match self.signed_request_id() {
            Some(request_id) => {
                self.firmware_status(out, now, SignedFirmwareStatus::InstallRebooting);
                self.installed_request = Some(request_id);
            }
            None => {
                self.firmware_status(out, now, SignedFirmwareStatus::Installed);
                self.security_event(out, now, FIRMWARE_UPDATED);
            }
        }
        self.firmware = None;
        self.boot_request.firmware_version = Some(boot_version);
        self.station.firmware_version = version;
        self.resetting = Some(ResetType::Soft);
    }

    fn firmware_status(&mut self, out: &mut Vec<Output>, now: u64, status: SignedFirmwareStatus) {
        let request_id = self.signed_request_id();
        match request_id {
            Some(_) => self.push_signed_status(status, request_id),
            None => {
                if let Some(status) = plain(status) {
                    let request = FirmwareStatusNotificationRequest { status };
                    self.outbox
                        .push("FirmwareStatusNotification", 0, json!(request));
                }
            }
        }
        self.emit(out, now, EventKind::FirmwareStatus { status, request_id });
    }

    fn push_signed_status(&mut self, status: SignedFirmwareStatus, request_id: Option<i32>) {
        let request = SignedFirmwareStatusNotificationRequest { status, request_id };
        self.outbox
            .push("SignedFirmwareStatusNotification", 0, json!(request));
    }

    fn signed_request_id(&self) -> Option<i32> {
        let signed = self.firmware.as_ref()?.signed.as_ref()?;
        Some(signed.request_id)
    }

    fn install_at(&self) -> u64 {
        let signed = self
            .firmware
            .as_ref()
            .and_then(|update| update.signed.as_ref());
        signed.map_or(0, |signed| signed.install_at)
    }

    fn no_transaction(&self) -> bool {
        self.transactions
            .iter()
            .all(|transaction| matches!(transaction, Transaction::Idle | Transaction::Finished))
    }
}

fn plain(status: SignedFirmwareStatus) -> Option<FirmwareStatus> {
    match status {
        SignedFirmwareStatus::Downloaded => Some(FirmwareStatus::Downloaded),
        SignedFirmwareStatus::DownloadFailed => Some(FirmwareStatus::DownloadFailed),
        SignedFirmwareStatus::Downloading => Some(FirmwareStatus::Downloading),
        SignedFirmwareStatus::Idle => Some(FirmwareStatus::Idle),
        SignedFirmwareStatus::InstallationFailed => Some(FirmwareStatus::InstallationFailed),
        SignedFirmwareStatus::Installing => Some(FirmwareStatus::Installing),
        SignedFirmwareStatus::Installed => Some(FirmwareStatus::Installed),
        _ => None,
    }
}

#[cfg(test)]
#[path = "firmware.test.rs"]
mod tests;
