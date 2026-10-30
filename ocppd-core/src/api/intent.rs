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
    SetCsms {
        url: String,
    },
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
            | Intent::SetCsms { .. }
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
    pub target_soc: f64,
    pub pack_v: f64,
    pub ac_kw: f64,
    pub ac_phases: u32,
    pub dc_kw: f64,
}

impl Default for EvSpec {
    fn default() -> Self {
        EvSpec {
            capacity_kwh: 60.0,
            soc: 0.2,
            target_soc: 1.0,
            pack_v: 400.0,
            ac_kw: 11.0,
            ac_phases: 3,
            dc_kw: 150.0,
        }
    }
}

impl EvSpec {
    pub fn ev(&self) -> Result<Ev, &'static str> {
        if self.capacity_kwh <= 0.0 || self.pack_v <= 0.0 || self.dc_kw <= 0.0 || self.ac_kw < 0.0 {
            return Err("capacity_kwh, pack_v and dc_kw must be positive, ac_kw not negative");
        }
        if !(1..=3).contains(&self.ac_phases) {
            return Err("ac_phases must be 1 to 3");
        }
        if !(0.0..=1.0).contains(&self.soc) || !(0.0..=1.0).contains(&self.target_soc) {
            return Err("soc and target_soc must be 0 to 1");
        }
        Ok(Ev {
            capacity_wh: self.capacity_kwh * 1000.0,
            pack_v: self.pack_v,
            ac_w: self.ac_kw * 1000.0,
            ac_phases: self.ac_phases,
            dc_w: self.dc_kw * 1000.0,
            target_soc: self.target_soc,
        })
    }
}
