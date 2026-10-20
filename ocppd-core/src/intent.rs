use crate::charge::Ev;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Intent {
    PlugIn {
        connector: u32,
        #[serde(default)]
        ev: EvSpec,
    },
    Authorize {
        connector: u32,
        id_tag: String,
    },
    Stop {
        connector: u32,
    },
    Unplug {
        connector: u32,
    },
}

impl Intent {
    pub fn connector(&self) -> u32 {
        match self {
            Intent::PlugIn { connector, .. }
            | Intent::Authorize { connector, .. }
            | Intent::Stop { connector }
            | Intent::Unplug { connector } => *connector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct EvSpec {
    pub capacity_kwh: f64,
    pub soc: f64,
    pub max_kw: f64,
    pub target_soc: f64,
}

impl Default for EvSpec {
    fn default() -> Self {
        EvSpec {
            capacity_kwh: 60.0,
            soc: 0.2,
            max_kw: 11.0,
            target_soc: 1.0,
        }
    }
}

impl EvSpec {
    pub fn ev(&self) -> Ev {
        Ev {
            capacity_wh: self.capacity_kwh * 1000.0,
            max_w: self.max_kw * 1000.0,
            target_soc: self.target_soc,
        }
    }
}
