use super::ChargingStation;
use super::registration::Registration;
use crate::api::io::Output;
use crate::v201::event::EventKind;
use crate::v201::messages::provisioning::boot::BootReason;
use crate::v201::messages::provisioning::reset::{ResetRequest, ResetResponse, ResetStatus};

impl ChargingStation {
    pub(super) fn reset(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ResetRequest,
    ) -> ResetResponse {
        let status = match request.evse_id {
            Some(_) => ResetStatus::Rejected,
            None => {
                self.resetting = true;
                let reset_type = request.kind;
                self.emit(out, now, EventKind::Reset { reset_type });
                ResetStatus::Accepted
            }
        };
        ResetResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn reset_due(&self, now: u64) -> bool {
        self.resetting && self.outbox.is_drained(now)
    }

    pub(super) fn rebooted(&mut self) {
        self.resetting = false;
        self.registration = Registration::Offline;
        self.boot_request.reason = BootReason::RemoteReset;
        self.outbox
            .keep(|action, _| super::TRANSACTION_MESSAGES.contains(&action));
        self.resend_statuses = true;
    }
}

#[cfg(test)]
#[path = "reset.test.rs"]
mod tests;
