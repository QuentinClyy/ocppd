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
    phases: Option<u32>,
    max_output_a: f64,
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
            phases: None,
            max_output_a: 500.0,
            current: Current::Ac,
            connectors: 1,
        }
    }
}

impl Model {
    fn phases(&self) -> u32 {
        self.phases.unwrap_or(match self.current {
            Current::Ac => 1,
            Current::Dc => 3,
        })
    }

    pub fn station(&self, id: &str, jitter: bool) -> Result<Station, &'static str> {
        let phases = self.phases();
        if !(1..=3).contains(&phases) {
            return Err("phases must be 1 to 3");
        }
        if self.current == Current::Dc && phases != 3 {
            return Err("dc needs phases 3");
        }
        if self.connectors == 0 {
            return Err("connectors must be at least 1");
        }
        if self.voltage_v <= 0.0 || self.max_current_a <= 0.0 || self.max_output_a <= 0.0 {
            return Err("voltage_v, max_current_a and max_output_a must be positive");
        }
        Ok(Station {
            id: id.into(),
            vendor: self.vendor.clone(),
            model: self.model.clone(),
            serial_number: id.into(),
            firmware_version: self.firmware_version.clone(),
            voltage_v: self.voltage_v,
            max_current_a: self.max_current_a,
            phases,
            max_output_a: self.max_output_a,
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
