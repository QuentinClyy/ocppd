use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::messages::transaction::{StoppedReason, TriggerReason};
use crate::station::connector::Fault;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn fault(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
    ) {
        self.station.connectors[index].fault = Some(Fault::Internal);
        let trigger = TriggerReason::AbnormalCondition;
        self.end_transaction(out, now, index, trigger, StoppedReason::Other);
    }
}

#[cfg(test)]
#[path = "fault.test.rs"]
mod tests;
