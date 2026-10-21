use super::ChargingStation;
use crate::api::io::Output;
use crate::station::jitter::fraction;
use crate::v201::event::EventKind;
use crate::v201::messages::availability::HeartbeatRequest;
use crate::v201::messages::provisioning::boot::{BootNotificationResponse, RegistrationStatus};
use serde_json::{Value, json};

const RETRY_MS: u64 = 60_000;

pub enum Registration {
    Offline,
    Disconnected {
        heartbeat_ms: Option<u64>,
    },
    Booting,
    Waiting {
        retry_at: u64,
    },
    Accepted {
        last_heartbeat: u64,
        heartbeat_ms: Option<u64>,
    },
}

impl ChargingStation {
    pub(super) fn is_connected(&self) -> bool {
        !matches!(
            self.registration,
            Registration::Offline | Registration::Disconnected { .. }
        )
    }

    pub(super) fn is_accepted(&self) -> bool {
        matches!(self.registration, Registration::Accepted { .. })
    }

    pub(super) fn connected(&mut self, now: u64) {
        match self.registration {
            Registration::Disconnected { heartbeat_ms } => {
                self.registration = Registration::Accepted {
                    last_heartbeat: now,
                    heartbeat_ms,
                }
            }
            _ => self.boot(),
        }
    }

    pub(super) fn disconnected(&mut self) {
        self.registration = match self.registration {
            Registration::Accepted { heartbeat_ms, .. } => {
                Registration::Disconnected { heartbeat_ms }
            }
            _ => Registration::Offline,
        };
    }

    fn boot(&mut self) {
        self.registration = Registration::Booting;
        self.outbox
            .push("BootNotification", 0, json!(self.boot_request));
    }

    pub(super) fn boot_failed(&mut self, now: u64) {
        self.registration = Registration::Waiting {
            retry_at: now + RETRY_MS,
        };
    }

    pub(super) fn booted(&mut self, out: &mut Vec<Output>, now: u64, payload: Value) {
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
            self.registration = Registration::Waiting {
                retry_at: now + wait_ms,
            };
            return;
        }
        let heartbeat_ms = Some(interval_ms).filter(|&ms| ms > 0);
        let phase = match self.station.jitter {
            true => fraction(&self.station.id, 0),
            false => 0.0,
        };
        let phase_ms = (phase * interval_ms as f64) as u64;
        self.registration = Registration::Accepted {
            last_heartbeat: now.saturating_sub(phase_ms),
            heartbeat_ms,
        };
        self.resend_statuses = true;
    }

    pub(super) fn registration_tick(&mut self, now: u64) {
        match self.registration {
            Registration::Waiting { retry_at } if now >= retry_at => self.boot(),
            Registration::Accepted {
                last_heartbeat,
                heartbeat_ms: Some(interval),
            } if now >= last_heartbeat + interval => {
                self.outbox.push("Heartbeat", 0, json!(HeartbeatRequest {}));
                self.registration = Registration::Accepted {
                    last_heartbeat: now,
                    heartbeat_ms: Some(interval),
                };
            }
            _ => {}
        }
    }

    pub(super) fn registration_deadline(&self) -> Option<u64> {
        match self.registration {
            Registration::Waiting { retry_at } => Some(retry_at),
            Registration::Accepted {
                last_heartbeat,
                heartbeat_ms,
            } => heartbeat_ms.map(|interval| last_heartbeat + interval),
            Registration::Offline | Registration::Disconnected { .. } | Registration::Booting => {
                None
            }
        }
    }
}

#[cfg(test)]
#[path = "registration.test.rs"]
mod tests;
