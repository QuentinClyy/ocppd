use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::provisioning::registration::Registration;
use crate::protocol::v201::charging_station::security::RESET_OR_REBOOT;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::provisioning::boot::BootReason;
use crate::protocol::v201::messages::provisioning::reset::{
    ResetRequest, ResetResponse, ResetStatus, ResetType,
};
use crate::protocol::v201::messages::transactions::{StoppedReason, TriggerReason};

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn reset(
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
        let running = self
            .sessions
            .iter()
            .any(|session| session.transaction.is_some());
        let reason = match (reset_type, running) {
            (ResetType::OnIdle, true) => BootReason::ScheduledReset,
            _ => BootReason::RemoteReset,
        };
        self.reboot.get_or_insert(reason);
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

    pub(in crate::protocol::v201::charging_station) fn power_cut(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
    ) {
        for index in 0..self.sessions.len() {
            let trigger = TriggerReason::AbnormalCondition;
            self.end_transaction(out, now, index, trigger, StoppedReason::PowerLoss);
        }
        self.reboot = Some(BootReason::PowerUp);
    }

    pub(in crate::protocol::v201::charging_station) fn shut_down(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
    ) {
        for index in 0..self.sessions.len() {
            let trigger = TriggerReason::AbnormalCondition;
            self.end_transaction(out, now, index, trigger, StoppedReason::Other);
        }
        self.shutting_down = true;
    }

    pub fn finished(&self, now: u64) -> bool {
        let idle = self
            .sessions
            .iter()
            .all(|session| session.transaction.is_none());
        self.shutting_down && idle && self.outbox.is_drained(now)
    }

    pub(in crate::protocol::v201::charging_station) fn reset_due(&self, now: u64) -> bool {
        let idle = self
            .sessions
            .iter()
            .all(|session| session.transaction.is_none());
        self.reboot.is_some() && idle && self.outbox.is_drained(now)
    }

    pub(in crate::protocol::v201::charging_station) fn rebooted(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
    ) {
        let Some(reason) = self.reboot.take() else {
            return;
        };
        self.registration = Registration::Offline;
        self.boot_request.reason = reason;
        self.outbox
            .keep(|action, _| crate::protocol::v201::charging_station::kept_offline(action));
        self.startup_reported = false;
        if reason != BootReason::PowerUp {
            self.security_event(out, now, RESET_OR_REBOOT);
        }
        self.resend_statuses = true;
        self.try_first_profile();
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
