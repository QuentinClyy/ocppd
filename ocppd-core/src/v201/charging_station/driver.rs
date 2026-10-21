use super::ChargingStation;
use crate::api::intent::Intent;
use crate::api::io::Output;

impl ChargingStation {
    pub(super) fn intent(&mut self, out: &mut Vec<Output>, now: u64, intent: Intent) {
        let evse = intent.connector();
        let index = evse
            .and_then(|evse| evse.checked_sub(1))
            .map(|index| index as usize)
            .filter(|&index| index < self.station.connectors.len());
        let Some(index) = index else {
            return self.reject(out, now, evse, "unknown connector");
        };
        let connector = &mut self.station.connectors[index];
        match intent {
            Intent::PlugIn { ev, .. } if connector.plugged.is_none() => {
                connector.plug_in(ev.ev(), ev.soc)
            }
            Intent::PlugIn { .. } => self.reject(out, now, evse, "already plugged in"),
            Intent::Unplug { .. } if connector.plugged.is_some() => connector.unplug(now),
            Intent::Unplug { .. } => self.reject(out, now, evse, "nothing plugged in"),
            Intent::Authorize { .. } | Intent::Stop { .. } => self.reject(
                out,
                now,
                evse,
                "transactions are not implemented for OCPP 2.0.1",
            ),
        }
    }
}
