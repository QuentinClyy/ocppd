use super::ChargePoint;
use super::transaction::Transaction;
use crate::api::io::Output;
use crate::protocol::v16::messages::core::transaction::Reason;
use crate::station::connector::Fault;

impl ChargePoint {
    pub(crate) fn fault(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        self.station.connectors[index].fault = Some(Fault::Internal);
        if matches!(self.transactions[index], Transaction::Active { .. }) {
            self.stop(out, now, index, Reason::Other);
        }
    }

    pub(crate) fn faulted(&self, index: usize) -> bool {
        self.station.connectors[index].fault.is_some()
    }
}

#[cfg(test)]
#[path = "fault.test.rs"]
mod tests;
