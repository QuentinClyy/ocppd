use ocppd_core::api::io::{Input, Output};
use ocppd_core::api::setting::Setting;
use ocppd_core::protocol::shared::x509::identity::ClientCertificate;
use ocppd_core::protocol::v16::charge_point::ChargePoint;
use ocppd_core::protocol::v16::messages::provisioning::configuration::ConfigurationStatus;
use ocppd_core::protocol::v201::charging_station::ChargingStation;
use ocppd_core::station::Station;
use ocppd_core::wire::ci_string::TooLong;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Copy, Default, Deserialize)]
pub enum Version {
    #[default]
    #[serde(rename = "1.6")]
    V16,
    #[serde(rename = "2.0.1")]
    V201,
}

#[allow(clippy::large_enum_variant)]
pub enum Protocol {
    V16(ChargePoint),
    V201(ChargingStation),
}

impl Protocol {
    pub fn new(version: Version, station: Station) -> Result<Protocol, TooLong> {
        Ok(match version {
            Version::V16 => Protocol::V16(ChargePoint::new(station)?),
            Version::V201 => Protocol::V201(ChargingStation::new(station)?),
        })
    }

    pub fn subprotocol(&self) -> &'static str {
        match self {
            Protocol::V16(_) => "ocpp1.6",
            Protocol::V201(_) => "ocpp2.0.1",
        }
    }

    pub fn station(&self) -> &Station {
        match self {
            Protocol::V16(charge_point) => charge_point.station(),
            Protocol::V201(charging_station) => charging_station.station(),
        }
    }

    pub fn finished(&self, now: u64) -> bool {
        match self {
            Protocol::V16(charge_point) => charge_point.finished(now),
            Protocol::V201(charging_station) => charging_station.finished(now),
        }
    }

    pub fn next_deadline(&self) -> Option<u64> {
        match self {
            Protocol::V16(charge_point) => charge_point.next_deadline(),
            Protocol::V201(charging_station) => charging_station.next_deadline(),
        }
    }

    pub fn reconnect_delay_ms(&self, failures: u32) -> Option<u64> {
        match self {
            Protocol::V16(_) => None,
            Protocol::V201(charging_station) => Some(charging_station.reconnect_delay_ms(failures)),
        }
    }

    pub fn handle(&mut self, now: u64, input: Input) -> Vec<Output> {
        match self {
            Protocol::V16(charge_point) => charge_point.handle(now, input),
            Protocol::V201(charging_station) => charging_station.handle(now, input),
        }
    }

    pub fn configure(&mut self, key: &str, value: &str) -> bool {
        match self {
            Protocol::V16(charge_point) => {
                charge_point.configure(key, value) == ConfigurationStatus::Accepted
            }
            Protocol::V201(charging_station) => charging_station.configure(key, value),
        }
    }

    pub fn settings(&mut self, now: u64) -> Vec<Setting> {
        match self {
            Protocol::V16(charge_point) => charge_point.settings(),
            Protocol::V201(charging_station) => charging_station.settings(now),
        }
    }

    pub fn description(&mut self, csms: &str, now: u64) -> String {
        let ocpp = match self {
            Protocol::V16(_) => "1.6",
            Protocol::V201(_) => "2.0.1",
        };
        let configuration = self.settings(now);
        let station = self.station();
        json!({
            "station": station.id,
            "at": now,
            "kind": "station_described",
            "ocpp": ocpp,
            "vendor": station.vendor,
            "model": station.model,
            "firmware_version": station.firmware_version,
            "csms": csms,
            "configuration": configuration,
        })
        .to_string()
    }

    pub fn set_basic_auth_password(&mut self, password: &str) -> bool {
        match self {
            Protocol::V16(charge_point) => charge_point.set_basic_auth_password(password),
            Protocol::V201(charging_station) => charging_station.set_basic_auth_password(password),
        }
    }

    pub fn basic_auth_password(&self) -> Option<Vec<u8>> {
        match self {
            Protocol::V16(charge_point) => charge_point.basic_auth_password(),
            Protocol::V201(charging_station) => charging_station.basic_auth_password(),
        }
    }

    pub fn requires_tls(&self) -> bool {
        match self {
            Protocol::V16(charge_point) => charge_point.requires_tls(),
            Protocol::V201(charging_station) => charging_station.requires_tls(),
        }
    }

    pub fn requires_client_certificate(&self) -> bool {
        match self {
            Protocol::V16(charge_point) => charge_point.requires_client_certificate(),
            Protocol::V201(charging_station) => charging_station.requires_client_certificate(),
        }
    }

    pub fn client_certificate(&self) -> Option<ClientCertificate> {
        match self {
            Protocol::V16(charge_point) => charge_point.client_certificate(),
            Protocol::V201(charging_station) => charging_station.client_certificate(),
        }
    }

    pub fn set_csms_url(&mut self, url: &str) -> bool {
        match self {
            Protocol::V16(_) => true,
            Protocol::V201(charging_station) => charging_station.set_csms_url(url),
        }
    }

    pub fn wants_connection(&self) -> bool {
        match self {
            Protocol::V16(charge_point) => charge_point.wants_connection(),
            Protocol::V201(charging_station) => charging_station.wants_connection(),
        }
    }

    pub fn csms_url(&self) -> Option<String> {
        match self {
            Protocol::V16(_) => None,
            Protocol::V201(charging_station) => charging_station.csms_url(),
        }
    }

    pub fn csms_roots(&self) -> Vec<Vec<u8>> {
        match self {
            Protocol::V16(charge_point) => charge_point.csms_roots(),
            Protocol::V201(charging_station) => charging_station.csms_roots(),
        }
    }
}

#[cfg(test)]
#[path = "protocol.test.rs"]
mod tests;
