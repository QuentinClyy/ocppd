use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::table::{
    HEARTBEAT_INTERVAL, OCPP_COMM_CTRLR,
};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::availability::HeartbeatRequest;
use crate::protocol::v201::messages::provisioning::boot::{
    BootNotificationResponse, RegistrationStatus,
};
use crate::station::jitter::fraction;
use serde_json::{Value, json};

const RETRY_MS: u64 = 60_000;

pub enum Registration {
    Offline,
    Disconnected,
    Booting,
    Waiting { retry_at: u64 },
    Rejected { retry_at: u64 },
    Accepted { last_heartbeat: u64 },
}

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn is_connected(&self) -> bool {
        !matches!(
            self.registration,
            Registration::Offline | Registration::Disconnected
        )
    }

    pub(in crate::protocol::v201::charging_station) fn is_accepted(&self) -> bool {
        matches!(self.registration, Registration::Accepted { .. })
    }

    pub(in crate::protocol::v201::charging_station) fn is_rejected(&self) -> bool {
        matches!(self.registration, Registration::Rejected { .. })
    }

    pub(in crate::protocol::v201::charging_station) fn connected(&mut self, now: u64) {
        match self.registration {
            Registration::Disconnected => {
                self.registration = Registration::Accepted {
                    last_heartbeat: now,
                }
            }
            _ => self.boot(),
        }
    }

    pub(in crate::protocol::v201::charging_station) fn disconnected(&mut self) {
        self.registration = match self.registration {
            Registration::Accepted { .. } => Registration::Disconnected,
            _ => Registration::Offline,
        };
    }

    fn boot(&mut self) {
        self.registration = Registration::Booting;
        self.outbox
            .push("BootNotification", 0, json!(self.boot_request));
    }

    pub(in crate::protocol::v201::charging_station) fn boot_failed(&mut self, now: u64) {
        self.registration = Registration::Waiting {
            retry_at: now + RETRY_MS,
        };
    }

    pub(in crate::protocol::v201::charging_station) fn booted(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        payload: Value,
    ) {
        let Some(response) =
            self.decode::<BootNotificationResponse>(out, now, "BootNotification", payload)
        else {
            return self.boot_failed(now);
        };
        let status = response.status;
        self.emit(out, now, EventKind::BootStatus { status });
        let interval_ms = u64::from(response.interval) * 1000;
        if status != RegistrationStatus::Accepted {
            let wait_ms = Some(interval_ms).filter(|&ms| ms > 0).unwrap_or(RETRY_MS);
            let retry_at = now + wait_ms;
            self.registration = match status {
                RegistrationStatus::Rejected => Registration::Rejected { retry_at },
                _ => Registration::Waiting { retry_at },
            };
            return;
        }
        let interval = response.interval.to_string();
        self.device_model
            .set_value(OCPP_COMM_CTRLR, None, HEARTBEAT_INTERVAL, interval);
        let phase = match self.station.jitter {
            true => fraction(&self.station.id, 0),
            false => 0.0,
        };
        let phase_ms = (phase * interval_ms as f64) as u64;
        self.registration = Registration::Accepted {
            last_heartbeat: now.saturating_sub(phase_ms),
        };
        self.resend_statuses = true;
        self.installed_after_reboot(out, now);
    }

    pub(in crate::protocol::v201::charging_station) fn registration_tick(&mut self, now: u64) {
        let heartbeat_due = self.heartbeat_at().is_some_and(|at| now >= at);
        match self.registration {
            Registration::Waiting { retry_at } | Registration::Rejected { retry_at }
                if now >= retry_at =>
            {
                self.boot()
            }
            Registration::Accepted { .. } if heartbeat_due => {
                self.outbox.push("Heartbeat", 0, json!(HeartbeatRequest {}));
                self.registration = Registration::Accepted {
                    last_heartbeat: now,
                };
            }
            _ => {}
        }
    }

    pub(in crate::protocol::v201::charging_station) fn registration_deadline(&self) -> Option<u64> {
        match self.registration {
            Registration::Waiting { retry_at } | Registration::Rejected { retry_at } => {
                Some(retry_at)
            }
            Registration::Accepted { .. } => self.heartbeat_at(),
            Registration::Offline | Registration::Disconnected | Registration::Booting => None,
        }
    }

    fn heartbeat_at(&self) -> Option<u64> {
        let Registration::Accepted { last_heartbeat } = self.registration else {
            return None;
        };
        let seconds = self
            .device_model
            .number(OCPP_COMM_CTRLR, HEARTBEAT_INTERVAL)
            .filter(|&seconds| seconds > 0)?;
        Some(last_heartbeat + seconds * 1000)
    }
}

#[cfg(test)]
#[path = "registration.test.rs"]
mod tests;
