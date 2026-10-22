use super::ChargePoint;
use super::transaction::Transaction;
use crate::api::io::Output;
use crate::protocol::v16::messages::core::transaction::Reason;

impl ChargePoint {
    pub(crate) fn fault(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        self.station.connectors[index].faulted = true;
        if matches!(self.transactions[index], Transaction::Active { .. }) {
            self.stop(out, now, index, Reason::Other);
        }
    }

    pub(crate) fn clear_fault(&mut self, index: usize) {
        self.station.connectors[index].faulted = false;
    }

    pub(crate) fn faulted(&self, index: usize) -> bool {
        self.station.connectors[index].faulted
    }
}

#[cfg(test)]
#[path = "fault.test.rs"]
mod tests;
