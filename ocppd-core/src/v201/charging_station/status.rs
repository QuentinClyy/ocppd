use super::ChargingStation;
use crate::api::io::Output;
use crate::v201::event::EventKind;
use crate::v201::messages::availability::StatusNotificationRequest;
use crate::wire::date_time::DateTime;
use serde_json::json;

const CONNECTOR_ID: u32 = 1;

impl ChargingStation {
    pub(super) fn report_statuses(&mut self, out: &mut Vec<Output>, now: u64) {
        for index in 0..self.reported.len() {
            let status = self.connector_status(index);
            let changed = self.reported[index] != Some(status);
            if !changed && !self.resend_statuses {
                continue;
            }
            self.reported[index] = Some(status);
            let evse = index as u32 + 1;
            let request = StatusNotificationRequest {
                timestamp: DateTime { unix_ms: now },
                connector_status: status,
                evse_id: evse,
                connector_id: CONNECTOR_ID,
            };
            self.outbox.push("StatusNotification", evse, json!(request));
            if changed {
                let connector = CONNECTOR_ID;
                self.emit(
                    out,
                    now,
                    EventKind::StatusChanged {
                        evse,
                        connector,
                        status,
                    },
                );
            }
        }
        self.resend_statuses = false;
    }
}

#[cfg(test)]
#[path = "status.test.rs"]
mod tests;
