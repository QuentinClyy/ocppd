use super::Authorization;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::table::TX_CTRLR;
use crate::protocol::v201::messages::transactions::{ChargingState, StoppedReason, TriggerReason};

#[derive(Clone, Copy, PartialEq)]
pub enum TxPoint {
    EvConnected,
    Authorized,
    PowerPathClosed,
    EnergyTransfer,
}

impl ChargingStation {
    pub(super) fn valid_points(&self, now: u64, index: usize) -> Vec<TxPoint> {
        let plugged = self.station.connectors[index].plugged.is_some();
        let authorized = matches!(
            self.sessions[index].authorization,
            Authorization::Authorized { .. }
        );
        let energy = self.charging_state(now, index) == ChargingState::Charging;
        [
            (TxPoint::EvConnected, plugged),
            (TxPoint::Authorized, authorized),
            (TxPoint::PowerPathClosed, plugged && authorized),
            (TxPoint::EnergyTransfer, energy),
        ]
        .into_iter()
        .filter_map(|(point, valid)| valid.then_some(point))
        .collect()
    }

    pub(super) fn points(&self, variable: &str) -> Vec<TxPoint> {
        let list = self
            .device_model
            .value(TX_CTRLR, variable)
            .unwrap_or_default();
        list.split(',')
            .filter_map(|point| match point {
                "EVConnected" => Some(TxPoint::EvConnected),
                "Authorized" => Some(TxPoint::Authorized),
                "PowerPathClosed" => Some(TxPoint::PowerPathClosed),
                "EnergyTransfer" => Some(TxPoint::EnergyTransfer),
                _ => None,
            })
            .collect()
    }
}

pub fn stopped_reason(trigger: TriggerReason, state: ChargingState) -> StoppedReason {
    match trigger {
        TriggerReason::StopAuthorized => StoppedReason::Local,
        TriggerReason::EvCommunicationLost => StoppedReason::EvDisconnected,
        TriggerReason::EvConnectTimeout => StoppedReason::Timeout,
        TriggerReason::Deauthorized => StoppedReason::DeAuthorized,
        TriggerReason::RemoteStop => StoppedReason::Remote,
        TriggerReason::ResetCommand => StoppedReason::ImmediateReset,
        TriggerReason::ChargingStateChanged if state == ChargingState::SuspendedEv => {
            StoppedReason::StoppedByEv
        }
        _ => StoppedReason::Other,
    }
}

#[cfg(test)]
#[path = "points.test.rs"]
mod tests;
