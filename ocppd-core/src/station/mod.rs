pub mod charge;
pub mod connector;
pub mod jitter;

use connector::Connector;

pub struct Station {
    pub id: String,
    pub vendor: String,
    pub model: String,
    pub serial_number: String,
    pub firmware_version: String,
    pub voltage_v: f64,
    pub max_current_a: f64,
    pub phases: u32,
    pub jitter: bool,
    pub connectors: Vec<Connector>,
}

impl Station {
    pub fn limit_w(&self) -> f64 {
        self.voltage_v * self.max_current_a * f64::from(self.phases)
    }
}
