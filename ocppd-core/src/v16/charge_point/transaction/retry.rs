use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::charge_point::ChargePoint;
use crate::v16::charge_point::configuration::{
    TRANSACTION_MESSAGE_ATTEMPTS, TRANSACTION_MESSAGE_RETRY_INTERVAL,
};
use crate::wire::outbox::{Sent, is_transaction_message};

impl ChargePoint {
    pub(crate) fn retry_transaction_message(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        sent: Sent,
    ) -> Option<Sent> {
        let action = sent.call.action.clone();
        if !is_transaction_message(&action) {
            return Some(sent);
        }
        let attempts = self
            .configuration
            .number(TRANSACTION_MESSAGE_ATTEMPTS)
            .unwrap_or(1);
        if sent.attempts >= attempts {
            self.emit(out, now, EventKind::CallAbandoned { action });
            return Some(sent);
        }
        let interval = self
            .configuration
            .interval_ms(TRANSACTION_MESSAGE_RETRY_INTERVAL)
            .unwrap_or(0);
        let retry_at = now + interval * u64::from(sent.attempts);
        let attempt = sent.attempts;
        self.emit(
            out,
            now,
            EventKind::CallRetried {
                action,
                attempt,
                retry_at,
            },
        );
        self.outbox.retry(sent, retry_at);
        None
    }
}

#[cfg(test)]
#[path = "retry.test.rs"]
mod tests;
