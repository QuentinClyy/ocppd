use super::registration::Registration;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::messages::provisioning::boot::HeartbeatRequest;
use serde_json::json;

impl ChargePoint {
    pub(crate) fn heartbeat(&mut self, now: u64) {
        self.outbox.push("Heartbeat", 0, json!(HeartbeatRequest {}));
        if self.is_accepted() {
            self.registration = Registration::Accepted {
                last_heartbeat: now,
            };
        }
    }
}

#[cfg(test)]
#[path = "heartbeat.test.rs"]
mod tests;
