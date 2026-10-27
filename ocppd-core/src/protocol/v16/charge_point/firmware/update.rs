use super::signed::Signed;
use crate::api::io::Output;
use crate::protocol::shared::file_transfer::{DOWNLOAD_MS, TransferStage, reachable};
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::security::security_event::FIRMWARE_UPDATED;
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::firmware::{
    FirmwareStatus, FirmwareStatusNotificationRequest, UpdateFirmwareRequest,
};
use crate::protocol::v16::messages::remote_control::ResetType;
use crate::protocol::v16::messages::security::signed_firmware::SignedFirmwareStatus;
use crate::wire::ci_string::CiString;
use serde_json::json;

const INSTALL_MS: u64 = 10_000;

pub struct FirmwareUpdate {
    pub(super) location: String,
    pub(super) signed: Option<Signed>,
    pub(super) stage: Stage,
}

#[derive(Clone, Copy)]
pub(super) enum Stage {
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

    pub(crate) fn report_firmware_status(&mut self) {
        let status = plain(self.current_firmware_status()).unwrap_or(FirmwareStatus::Idle);
        let request = FirmwareStatusNotificationRequest { status };
        self.outbox
            .push("FirmwareStatusNotification", 0, json!(request));
    }

    pub(super) fn current_firmware_status(&self) -> SignedFirmwareStatus {
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

    pub(super) fn firmware_status(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        status: SignedFirmwareStatus,
    ) {
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
#[path = "update.test.rs"]
mod tests;
