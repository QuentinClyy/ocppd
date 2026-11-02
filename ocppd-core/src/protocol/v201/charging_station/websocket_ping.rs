use super::ChargingStation;
use super::device_model::table::OCPP_COMM_CTRLR;
use crate::api::io::Output;
use crate::protocol::shared::ping::{self, WEB_SOCKET_PING_INTERVAL};

impl ChargingStation {
    pub(super) fn ping_deadline(&self) -> Option<u64> {
        self.next_ping
    }

    pub(super) fn ping_tick(&mut self, out: &mut Vec<Output>, now: u64) {
        let seconds = self
            .device_model
            .number(OCPP_COMM_CTRLR, WEB_SOCKET_PING_INTERVAL)
            .unwrap_or(0);
        let (next, due) = ping::next(self.next_ping, now, seconds, self.is_connected());
        self.next_ping = next;
        if due {
            out.push(Output::Ping);
        }
    }
}

#[cfg(test)]
#[path = "websocket_ping.test.rs"]
mod tests;
