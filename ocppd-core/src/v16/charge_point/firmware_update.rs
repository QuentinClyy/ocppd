use super::ChargePoint;
use super::security_event::FIRMWARE_UPDATED;
use super::transaction::Transaction;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::messages::core::remote_control::ResetType;
use crate::v16::messages::firmware::{
    FirmwareStatus, FirmwareStatusNotificationRequest, UpdateFirmwareRequest,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

const DOWNLOAD_MS: u64 = 10_000;
const INSTALL_MS: u64 = 10_000;
const SCHEMES: [&str; 4] = ["http://", "https://", "ftp://", "ftps://"];

pub struct FirmwareUpdate {
    location: String,
    stage: Stage,
}

#[derive(Clone, Copy)]
enum Stage {
    Waiting { until: u64 },
    Downloading { until: u64 },
    Downloaded,
    Installing { until: u64 },
}

impl ChargePoint {
    pub(super) fn update_firmware(&mut self, now: u64, request: UpdateFirmwareRequest) {
        let until = request.retrieve_date.unix_ms.max(now);
        self.firmware = Some(FirmwareUpdate {
            location: request.location,
            stage: Stage::Waiting { until },
        });
    }

    pub(super) fn advance_firmware(&mut self, out: &mut Vec<Output>, now: u64) {
        while let Some(update) = &self.firmware {
            let next = match update.stage {
                Stage::Waiting { until } if now >= until => {
                    if !reachable(&update.location) {
                        self.firmware = None;
                        self.firmware_status(out, now, FirmwareStatus::DownloadFailed);
                        return;
                    }
                    self.firmware_status(out, now, FirmwareStatus::Downloading);
                    Stage::Downloading {
                        until: until + DOWNLOAD_MS,
                    }
                }
                Stage::Downloading { until } if now >= until => {
                    self.firmware_status(out, now, FirmwareStatus::Downloaded);
                    Stage::Downloaded
                }
                Stage::Downloaded if self.no_transaction() => {
                    self.firmware_status(out, now, FirmwareStatus::Installing);
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

    pub(super) fn firmware_deadline(&self) -> Option<u64> {
        match self.firmware.as_ref()?.stage {
            Stage::Waiting { until }
            | Stage::Downloading { until }
            | Stage::Installing { until } => Some(until),
            Stage::Downloaded => None,
        }
    }

    pub(super) fn current_firmware_status(&self) -> FirmwareStatus {
        match self.firmware.as_ref().map(|update| update.stage) {
            None | Some(Stage::Waiting { .. }) => FirmwareStatus::Idle,
            Some(Stage::Downloading { .. }) => FirmwareStatus::Downloading,
            Some(Stage::Downloaded) => FirmwareStatus::Downloaded,
            Some(Stage::Installing { .. }) => FirmwareStatus::Installing,
        }
    }

    pub(super) fn firmware_status(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        status: FirmwareStatus,
    ) {
        let request = FirmwareStatusNotificationRequest { status };
        self.outbox
            .push("FirmwareStatusNotification", 0, json!(request));
        self.emit(out, now, EventKind::FirmwareStatus { status });
    }

    fn install_firmware(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(update) = self.firmware.take() else {
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
        let Some(boot_version) = boot_version else {
            return self.firmware_status(out, now, FirmwareStatus::InstallationFailed);
        };
        self.boot_request.firmware_version = Some(boot_version);
        self.station.firmware_version = version;
        self.firmware_status(out, now, FirmwareStatus::Installed);
        self.security_event(out, now, FIRMWARE_UPDATED);
        self.resetting = Some(ResetType::Soft);
    }

    fn no_transaction(&self) -> bool {
        self.transactions
            .iter()
            .all(|transaction| matches!(transaction, Transaction::Idle | Transaction::Finished))
    }
}

pub fn reachable(location: &str) -> bool {
    SCHEMES.iter().any(|scheme| location.starts_with(scheme))
}

#[cfg(test)]
#[path = "firmware_update.test.rs"]
mod tests;
