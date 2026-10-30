use ocppd_core::station::Station;
use ocppd_core::station::charge::Current;
use ocppd_core::station::connector::Connector;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Model {
    vendor: String,
    model: String,
    firmware_version: String,
    voltage_v: f64,
    max_current_a: f64,
    phases: u32,
    current: Current,
    connectors: usize,
}

impl Default for Model {
    fn default() -> Model {
        Model {
            vendor: "ocppd".into(),
            model: "sim".into(),
            firmware_version: env!("CARGO_PKG_VERSION").into(),
            voltage_v: 230.0,
            max_current_a: 32.0,
            phases: 3,
            current: Current::Ac,
            connectors: 1,
        }
    }
}

impl Model {
    pub fn station(&self, id: &str, jitter: bool) -> Result<Station, &'static str> {
        if !(1..=3).contains(&self.phases) {
            return Err("phases must be 1 to 3");
        }
        if self.current == Current::Dc && self.phases != 3 {
            return Err("dc needs phases 3");
        }
        if self.connectors == 0 {
            return Err("connectors must be at least 1");
        }
        if self.voltage_v <= 0.0 || self.max_current_a <= 0.0 {
            return Err("voltage_v and max_current_a must be positive");
        }
        Ok(Station {
            id: id.into(),
            vendor: self.vendor.clone(),
            model: self.model.clone(),
            serial_number: id.into(),
            firmware_version: self.firmware_version.clone(),
            voltage_v: self.voltage_v,
            max_current_a: self.max_current_a,
            phases: self.phases,
            jitter,
            connectors: vec![
                Connector {
                    current: self.current,
                    ..Connector::default()
                };
                self.connectors
            ],
        })
    }
}

#[cfg(test)]
#[path = "model.test.rs"]
mod tests;
