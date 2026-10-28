use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::provisioning::configuration::HEARTBEAT_INTERVAL;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::provisioning::boot::{
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
    Accepted { last_heartbeat: u64 },
}

impl ChargePoint {
    pub(crate) fn is_connected(&self) -> bool {
        !matches!(
            self.registration,
            Registration::Offline | Registration::Disconnected
        )
    }

    pub fn csms_url(&self) -> &str {
        &self.csms_url
    }

    pub fn set_csms_url(&mut self, url: &str) {
        self.csms_url = url.to_owned();
    }

    pub(crate) fn change_csms(&mut self, out: &mut Vec<Output>, now: u64, url: &str) {
        if self.is_connected() {
            return self.reject(out, now, None, "disconnect first");
        }
        self.set_csms_url(url);
        let url = url.to_owned();
        self.emit(out, now, EventKind::Link(LinkEvent::CsmsChanged { url }));
    }

    pub fn wants_connection(&self) -> bool {
        !self.held_offline
    }

    pub(crate) fn hold_offline(&mut self, out: &mut Vec<Output>, now: u64) {
        if self.held_offline {
            return;
        }
        self.held_offline = true;
        self.emit(out, now, EventKind::Link(LinkEvent::Offline));
        out.push(Output::Close);
    }

    pub(crate) fn release(&mut self, out: &mut Vec<Output>, now: u64) {
        if !self.held_offline {
            return;
        }
        self.held_offline = false;
        self.emit(out, now, EventKind::Link(LinkEvent::Online));
    }

    pub(crate) fn is_accepted(&self) -> bool {
        matches!(self.registration, Registration::Accepted { .. })
    }

    pub(crate) fn connected(&mut self, now: u64) {
        match self.registration {
            Registration::Disconnected => {
                self.registration = Registration::Accepted {
                    last_heartbeat: now,
                }
            }
            _ => self.boot(),
        }
    }

    pub(crate) fn disconnected(&mut self) {
        self.registration = match self.registration {
            Registration::Accepted { .. } => Registration::Disconnected,
            _ => Registration::Offline,
        };
    }

    pub(crate) fn boot(&mut self) {
        self.registration = Registration::Booting;
        self.outbox
            .push("BootNotification", 0, json!(self.boot_request));
    }

    pub(crate) fn boot_failed(&mut self, now: u64) {
        self.registration = Registration::Waiting {
            retry_at: now + RETRY_MS,
        };
    }

    pub(crate) fn booted(&mut self, out: &mut Vec<Output>, now: u64, payload: Value) {
        let Some(response) =
            self.decode::<BootNotificationResponse>(out, now, "BootNotification", payload)
        else {
            return self.boot_failed(now);
        };
        let status = response.status;
        self.emit(out, now, EventKind::BootStatus { status });
        let interval_ms = match response.interval {
            0 => RETRY_MS,
            seconds => u64::from(seconds) * 1000,
        };
        if status != RegistrationStatus::Accepted {
            self.registration = Registration::Waiting {
                retry_at: now + interval_ms,
            };
            return;
        }
        let phase = match self.station.jitter {
            true => fraction(&self.station.id, 0),
            false => 0.0,
        };
        let phase_ms = (phase * interval_ms as f64) as u64;
        self.registration = Registration::Accepted {
            last_heartbeat: now.saturating_sub(phase_ms),
        };
        let seconds = (interval_ms / 1000) as u32;
        self.configuration.set_seconds(HEARTBEAT_INTERVAL, seconds);
        self.resend_statuses = true;
        self.installed_after_reboot(out, now);
    }

    pub(crate) fn registration_tick(&mut self, now: u64) {
        let heartbeat_interval = self.configuration.interval_ms(HEARTBEAT_INTERVAL);
        match (&self.registration, heartbeat_interval) {
            (Registration::Waiting { retry_at }, _) if now >= *retry_at => self.boot(),
            (Registration::Accepted { last_heartbeat }, Some(interval))
                if now >= last_heartbeat + interval =>
            {
                self.heartbeat(now)
            }
            _ => {}
        }
    }

    pub(crate) fn registration_deadline(&self) -> Option<u64> {
        match self.registration {
            Registration::Waiting { retry_at } => Some(retry_at),
            Registration::Accepted { last_heartbeat } => self
                .configuration
                .interval_ms(HEARTBEAT_INTERVAL)
                .map(|interval| last_heartbeat + interval),
            Registration::Offline | Registration::Disconnected | Registration::Booting => None,
        }
    }
}

#[cfg(test)]
#[path = "registration.test.rs"]
mod tests;
