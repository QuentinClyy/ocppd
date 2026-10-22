use super::ChargingStation;
use super::device_model::table::{STOP_TX_ON_EV_SIDE_DISCONNECT, TX_CTRLR};
use super::transaction::Authorization;
use crate::api::intent::Intent;
use crate::api::io::Output;
use crate::protocol::v201::messages::transactions::TriggerReason;
use crate::station::connector::Fault;

impl ChargingStation {
    pub(super) fn intent(&mut self, out: &mut Vec<Output>, now: u64, intent: Intent) {
        if let Intent::Authorize { connector, id_tag } = &intent {
            return self.present_id_token(out, now, *connector, id_tag);
        }
        let evse = intent.connector();
        let index = evse
            .and_then(|evse| evse.checked_sub(1))
            .map(|index| index as usize)
            .filter(|&index| index < self.station.connectors.len());
        let Some(index) = index else {
            return self.reject(out, now, evse, "unknown connector");
        };
        let plugged = self.station.connectors[index].plugged.is_some();
        match intent {
            Intent::PlugIn { .. } if plugged => self.reject(out, now, evse, "already plugged in"),
            Intent::PlugIn { ev, .. } => {
                self.station.connectors[index].plug_in(ev.ev(), ev.soc);
                if self.station.connectors[index].fault == Some(Fault::ConnectorLock) {
                    self.lock_failed(now, index);
                }
                self.settle(out, now, index, TriggerReason::CablePluggedIn);
            }
            Intent::Unplug { .. } if !plugged => self.reject(out, now, evse, "nothing plugged in"),
            Intent::Unplug { .. } => {
                self.station.connectors[index].unplug(now);
                if self
                    .device_model
                    .value(TX_CTRLR, STOP_TX_ON_EV_SIDE_DISCONNECT)
                    == Some("true")
                {
                    self.sessions[index].authorization = Authorization::None;
                }
                self.settle(out, now, index, TriggerReason::EvCommunicationLost);
            }
            Intent::Stop { .. } => {
                let session = &self.sessions[index];
                if session.transaction.is_none() && session.authorization == Authorization::None {
                    return self.reject(out, now, evse, "no active transaction");
                }
                self.sessions[index].authorization = Authorization::None;
                self.settle(out, now, index, TriggerReason::StopAuthorized);
            }
            Intent::Fault { .. } => self.fault(out, now, index),
            Intent::ClearFault { .. } => {
                self.station.connectors[index].repair();
                self.lock_repaired(index);
            }
            Intent::LockFailure { .. } => self.station.connectors[index].lock_broken = true,
            Intent::Authorize { .. } => {}
        }
    }
}
