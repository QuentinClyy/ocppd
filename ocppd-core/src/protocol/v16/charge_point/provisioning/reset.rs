use crate::api::io::Output;
use crate::protocol::v16::charge_point::provisioning::registration::Registration;
use crate::protocol::v16::charge_point::security::security_event::RESET_OR_REBOOT;
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::charge_point::{ChargePoint, survives_disconnect};
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v16::messages::remote_control::{ResetRequest, ResetType};
use crate::protocol::v16::messages::transaction::Reason;

impl ChargePoint {
    pub(crate) fn reset(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ResetRequest,
    ) -> GenericStatus {
        let reason = match request.kind {
            ResetType::Soft => Reason::SoftReset,
            ResetType::Hard => Reason::HardReset,
        };
        self.stop_all(out, now, reason);
        self.emit(
            out,
            now,
            EventKind::Reset {
                reset_type: request.kind,
            },
        );
        self.security_event(out, now, RESET_OR_REBOOT);
        self.resetting = Some(request.kind);
        GenericStatus::Accepted
    }

    pub(crate) fn power_cut(&mut self, out: &mut Vec<Output>, now: u64) {
        self.stop_all(out, now, Reason::PowerLoss);
        self.resetting = Some(ResetType::Soft);
    }

    pub(crate) fn shut_down(&mut self, out: &mut Vec<Output>, now: u64) {
        self.stop_all(out, now, Reason::Other);
        self.shutting_down = true;
    }

    pub fn finished(&self, now: u64) -> bool {
        self.shutting_down && self.outbox.is_drained(now)
    }

    fn stop_all(&mut self, out: &mut Vec<Output>, now: u64, reason: Reason) {
        for index in 0..self.transactions.len() {
            match self.transactions[index] {
                Transaction::Authorizing { .. } | Transaction::Starting { .. } => {
                    self.transactions[index] = Transaction::Idle
                }
                _ => self.stop(out, now, index, reason),
            }
        }
        self.held = None;
    }

    pub(crate) fn reset_due(&self, now: u64) -> bool {
        match self.resetting {
            Some(ResetType::Hard) => true,
            Some(ResetType::Soft) => self.outbox.is_drained(now),
            None => false,
        }
    }

    pub(crate) fn rebooted(&mut self) {
        self.resetting = None;
        self.startup_reported = false;
        self.registration = Registration::Offline;
        self.outbox.keep(survives_disconnect);
        self.connection_lost();
        for transaction in &mut self.transactions {
            if *transaction == Transaction::Finished {
                *transaction = Transaction::Idle;
            }
        }
    }
}

#[cfg(test)]
#[path = "reset.test.rs"]
mod tests;
