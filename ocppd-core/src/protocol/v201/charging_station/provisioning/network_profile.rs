use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::table::OCPP_COMM_CTRLR;
use crate::protocol::v201::charging_station::security::{SECURITY_CTRLR, SECURITY_PROFILE};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::status_info::StatusInfo;
use crate::protocol::v201::messages::provisioning::network_profile::{
    NetworkConnectionProfile, OcppInterface, OcppTransport, OcppVersion, SetNetworkProfileRequest,
    SetNetworkProfileResponse, SetNetworkProfileStatus,
};
use crate::wire::ci_string::CiString;

pub const NETWORK_CONFIGURATION_PRIORITY: &str = "NetworkConfigurationPriority";
pub const NETWORK_PROFILE_CONNECTION_ATTEMPTS: &str = "NetworkProfileConnectionAttempts";
pub const ACTIVE_NETWORK_PROFILE: &str = "ActiveNetworkProfile";
pub const SLOTS: &str = "0,1";
const SLOT_COUNT: usize = 2;
const MESSAGE_TIMEOUT_S: i32 = 30;
const TLS_FROM_PROFILE: i32 = 2;
const HIGHEST_PROFILE: i32 = 3;

#[derive(Default)]
pub struct NetworkProfiles {
    slots: [Option<NetworkConnectionProfile>; SLOT_COUNT],
    trying: Option<Trying>,
    last_connected: Option<usize>,
}

struct Trying {
    slot: usize,
    url: String,
    security_profile: u64,
    failures: u64,
}

impl ChargingStation {
    pub fn set_csms_url(&mut self, url: &str) -> bool {
        let Ok(url) = CiString::new(url) else {
            return false;
        };
        self.network.slots[0] = Some(NetworkConnectionProfile {
            apn: None,
            ocpp_version: OcppVersion::Ocpp20,
            ocpp_transport: OcppTransport::Json,
            ocpp_csms_url: url,
            message_timeout: MESSAGE_TIMEOUT_S,
            security_profile: self.current_security_profile() as i32,
            ocpp_interface: OcppInterface::Wired0,
            vpn: None,
        });
        self.try_first_profile();
        true
    }

    pub fn csms_url(&self) -> Option<String> {
        self.network
            .trying
            .as_ref()
            .map(|trying| trying.url.clone())
    }

    pub(in crate::protocol::v201::charging_station) fn connecting_security_profile(&self) -> u64 {
        match &self.network.trying {
            Some(trying) => trying.security_profile,
            None => self.current_security_profile(),
        }
    }

    pub(in crate::protocol::v201::charging_station) fn set_network_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: SetNetworkProfileRequest,
    ) -> SetNetworkProfileResponse {
        let slot = usize::try_from(request.configuration_slot)
            .ok()
            .filter(|&slot| slot < SLOT_COUNT);
        let Some(slot) = slot else {
            return response(SetNetworkProfileStatus::Rejected, Some("InvalidConfSlot"));
        };
        let profile = request.connection_data;
        if !valid(&profile) {
            return response(
                SetNetworkProfileStatus::Rejected,
                Some("InvalidNetworkConf"),
            );
        }
        if (profile.security_profile as u64) < self.current_security_profile() {
            return response(
                SetNetworkProfileStatus::Rejected,
                Some("NoSecurityDowngrade"),
            );
        }
        let security_profile = profile.security_profile as u64;
        self.network.slots[slot] = Some(profile);
        let kind = EventKind::NetworkProfileSet {
            slot,
            security_profile,
        };
        self.emit(out, now, kind);
        response(SetNetworkProfileStatus::Accepted, None)
    }

    pub(in crate::protocol::v201::charging_station) fn priority_allowed(
        &self,
        value: &str,
    ) -> bool {
        let current = self.current_security_profile();
        let has_roots = !self.csms_roots().is_empty();
        let has_certificate = self.has_client_certificate();
        value.split(',').all(|slot| {
            let profile = slot
                .parse::<usize>()
                .ok()
                .and_then(|slot| self.network.slots.get(slot)?.as_ref());
            let Some(profile) = profile else {
                return false;
            };
            let wanted = profile.security_profile as u64;
            let upgrade = wanted > current;
            wanted >= current
                && !(upgrade && wanted >= TLS_FROM_PROFILE as u64 && !has_roots)
                && !(upgrade && wanted == HIGHEST_PROFILE as u64 && !has_certificate)
        })
    }

    pub(in crate::protocol::v201::charging_station) fn network_connected(&mut self) -> bool {
        let Some(trying) = &mut self.network.trying else {
            return false;
        };
        trying.failures = 0;
        let (slot, security_profile) = (trying.slot, trying.security_profile);
        let switched = self.network.last_connected.is_some_and(|last| last != slot);
        self.network.last_connected = Some(slot);
        self.device_model.set_value(
            SECURITY_CTRLR,
            None,
            SECURITY_PROFILE,
            security_profile.to_string(),
        );
        self.device_model.set_value(
            OCPP_COMM_CTRLR,
            None,
            ACTIVE_NETWORK_PROFILE,
            slot.to_string(),
        );
        for other in &mut self.network.slots {
            if other
                .as_ref()
                .is_some_and(|other| (other.security_profile as u64) < security_profile)
            {
                *other = None;
            }
        }
        let kept: Vec<String> = self
            .priority()
            .into_iter()
            .filter(|&slot| self.network.slots[slot].is_some())
            .map(|slot| slot.to_string())
            .collect();
        self.device_model.set_value(
            OCPP_COMM_CTRLR,
            None,
            NETWORK_CONFIGURATION_PRIORITY,
            kept.join(","),
        );
        switched
    }

    pub(in crate::protocol::v201::charging_station) fn connect_failed(&mut self) {
        let attempts = self
            .device_model
            .number(OCPP_COMM_CTRLR, NETWORK_PROFILE_CONNECTION_ATTEMPTS)
            .unwrap_or(0)
            .max(1);
        let Some(trying) = &mut self.network.trying else {
            return;
        };
        trying.failures += 1;
        if trying.failures < attempts {
            return;
        }
        let current = trying.slot;
        let priority = self.priority();
        let position = priority.iter().position(|&slot| slot == current);
        let next = match position {
            Some(position) => priority[(position + 1) % priority.len()],
            None => priority.first().copied().unwrap_or(current),
        };
        self.try_profile(next);
    }

    pub(in crate::protocol::v201::charging_station) fn sync_first_profile(&mut self) {
        let current = self.current_security_profile() as i32;
        if let Some(profile) = &mut self.network.slots[0] {
            profile.security_profile = current;
        }
        self.try_first_profile();
    }

    pub(in crate::protocol::v201::charging_station) fn try_first_profile(&mut self) {
        if let Some(&first) = self.priority().first() {
            self.try_profile(first);
        }
    }

    fn try_profile(&mut self, slot: usize) {
        let Some(profile) = &self.network.slots[slot] else {
            return;
        };
        self.network.trying = Some(Trying {
            slot,
            url: profile.ocpp_csms_url.as_str().to_owned(),
            security_profile: profile.security_profile as u64,
            failures: 0,
        });
    }

    fn priority(&self) -> Vec<usize> {
        let value = self
            .device_model
            .value(OCPP_COMM_CTRLR, NETWORK_CONFIGURATION_PRIORITY)
            .unwrap_or_default();
        value
            .split(',')
            .filter_map(|slot| slot.parse::<usize>().ok())
            .filter(|&slot| self.network.slots.get(slot).is_some_and(Option::is_some))
            .collect()
    }

    fn current_security_profile(&self) -> u64 {
        self.device_model
            .number(SECURITY_CTRLR, SECURITY_PROFILE)
            .unwrap_or(0)
    }
}

fn valid(profile: &NetworkConnectionProfile) -> bool {
    let url = profile.ocpp_csms_url.as_str();
    let secure = url.starts_with("wss://");
    let scheme = secure || url.starts_with("ws://");
    let level = (0..=HIGHEST_PROFILE).contains(&profile.security_profile);
    profile.ocpp_version == OcppVersion::Ocpp20
        && profile.ocpp_transport == OcppTransport::Json
        && scheme
        && level
        && (secure || profile.security_profile < TLS_FROM_PROFILE)
}

fn response(status: SetNetworkProfileStatus, reason: Option<&str>) -> SetNetworkProfileResponse {
    let status_info = reason.and_then(|reason| {
        Some(StatusInfo {
            reason_code: CiString::new(reason).ok()?,
            additional_info: None,
        })
    });
    SetNetworkProfileResponse {
        status,
        status_info,
    }
}

#[cfg(test)]
#[path = "network_profile.test.rs"]
mod tests;
