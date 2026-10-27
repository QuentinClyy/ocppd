use super::update::{FirmwareUpdate, Stage};
use crate::api::io::Output;
use crate::protocol::shared::file_transfer::TransferStage;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::security::security_event::{
    FIRMWARE_UPDATED, INVALID_FIRMWARE_SIGNATURE, INVALID_FIRMWARE_SIGNING_CERTIFICATE,
};
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::security::signed_firmware::{
    SignedFirmwareStatus, SignedFirmwareStatusNotificationRequest, SignedUpdateFirmwareRequest,
    UpdateFirmwareStatus,
};
use serde_json::json;

impl ChargePoint {
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

    pub(crate) fn report_signed_firmware_status(&mut self) {
        let status = self.current_firmware_status();
        self.push_signed_status(status, self.signed_request_id());
    }

    pub(super) fn verify_signature(&mut self, out: &mut Vec<Output>, now: u64) -> bool {
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

    pub(super) fn push_signed_status(
        &mut self,
        status: SignedFirmwareStatus,
        request_id: Option<i32>,
    ) {
        let request = SignedFirmwareStatusNotificationRequest { status, request_id };
        self.outbox
            .push("SignedFirmwareStatusNotification", 0, json!(request));
    }

    pub(super) fn signed_request_id(&self) -> Option<i32> {
        let signed = self.firmware.as_ref()?.signed.as_ref()?;
        Some(signed.request_id)
    }

    pub(super) fn install_at(&self) -> u64 {
        let signed = self
            .firmware
            .as_ref()
            .and_then(|update| update.signed.as_ref());
        signed.map_or(0, |signed| signed.install_at)
    }
}

pub(super) struct Signed {
    request_id: i32,
    has_signature: bool,
    install_at: u64,
}

#[cfg(test)]
#[path = "signed.test.rs"]
mod tests;
