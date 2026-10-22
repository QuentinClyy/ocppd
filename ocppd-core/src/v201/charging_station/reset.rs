use super::ChargingStation;
use super::registration::Registration;
use super::security::RESET_OR_REBOOT;
use crate::api::io::Output;
use crate::v201::event::EventKind;
use crate::v201::messages::provisioning::boot::BootReason;
use crate::v201::messages::provisioning::reset::{
    ResetRequest, ResetResponse, ResetStatus, ResetType,
};
use crate::v201::messages::transactions::{StoppedReason, TriggerReason};

impl ChargingStation {
    pub(super) fn reset(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ResetRequest,
    ) -> ResetResponse {
        if request.evse_id.is_some() {
            return response(ResetStatus::Rejected);
        }
        let reset_type = request.kind;
        self.emit(out, now, EventKind::Reset { reset_type });
        self.resetting = true;
        let running = self
            .sessions
            .iter()
            .any(|session| session.transaction.is_some());
        match (reset_type, running) {
            (ResetType::OnIdle, true) => response(ResetStatus::Scheduled),
            (ResetType::Immediate, true) => {
                for index in 0..self.sessions.len() {
                    let trigger = TriggerReason::ResetCommand;
                    self.end_transaction(out, now, index, trigger, StoppedReason::ImmediateReset);
                }
                response(ResetStatus::Accepted)
            }
            (_, false) => response(ResetStatus::Accepted),
        }
    }

    pub(super) fn reset_due(&self, now: u64) -> bool {
        let idle = self
            .sessions
            .iter()
            .all(|session| session.transaction.is_none());
        self.resetting && idle && self.outbox.is_drained(now)
    }

    pub(super) fn rebooted(&mut self, out: &mut Vec<Output>, now: u64) {
        self.resetting = false;
        self.registration = Registration::Offline;
        self.boot_request.reason = match self.rebooting_for_firmware() {
            true => BootReason::FirmwareUpdate,
            false => BootReason::RemoteReset,
        };
        self.outbox.keep(|action, _| super::kept_offline(action));
        self.startup_reported = false;
        self.security_event(out, now, RESET_OR_REBOOT);
        self.resend_statuses = true;
    }
}

fn response(status: ResetStatus) -> ResetResponse {
    ResetResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "reset.test.rs"]
mod tests;
