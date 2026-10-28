use crate::protocol::shared::file_transfer::TransferStage;
use crate::station::charge::Ev;
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
        #[serde(default)]
        connector: Option<u32>,
        id_tag: String,
    },
    Stop {
        connector: u32,
    },
    Unplug {
        connector: u32,
    },
    Fault {
        connector: u32,
    },
    ClearFault {
        connector: u32,
    },
    LockFailure {
        connector: u32,
    },
    PowerCut,
    Disconnect,
    Connect,
    #[serde(skip_deserializing)]
    Shutdown,
    FailTransfer {
        stage: TransferStage,
    },
    Configure {
        key: String,
        value: String,
    },
}

impl Intent {
    pub fn connector(&self) -> Option<u32> {
        match self {
            Intent::Authorize { connector, .. } => *connector,
            Intent::PowerCut
            | Intent::Disconnect
            | Intent::Connect
            | Intent::Shutdown
            | Intent::FailTransfer { .. }
            | Intent::Configure { .. } => None,
            Intent::PlugIn { connector, .. }
            | Intent::Stop { connector }
            | Intent::Unplug { connector }
            | Intent::Fault { connector }
            | Intent::ClearFault { connector }
            | Intent::LockFailure { connector } => Some(*connector),
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
