use super::ChargingStation;
use super::device_model::table::OCPP_COMM_CTRLR;
use crate::api::io::Output;

pub const WEB_SOCKET_PING_INTERVAL: &str = "WebSocketPingInterval";

impl ChargingStation {
    pub(super) fn ping_deadline(&self) -> Option<u64> {
        self.next_ping
    }

    pub(super) fn ping_tick(&mut self, out: &mut Vec<Output>, now: u64) {
        let seconds = self
            .device_model
            .number(OCPP_COMM_CTRLR, WEB_SOCKET_PING_INTERVAL)
            .unwrap_or(0);
        if !self.is_connected() || seconds == 0 {
            self.next_ping = None;
            return;
        }
        let interval = seconds * 1000;
        match self.next_ping {
            Some(at) if at <= now => {
                out.push(Output::Ping);
                self.next_ping = Some(now + interval);
            }
            Some(_) => {}
            None => self.next_ping = Some(now + interval),
        }
    }
}

#[cfg(test)]
#[path = "websocket_ping.test.rs"]
mod tests;
