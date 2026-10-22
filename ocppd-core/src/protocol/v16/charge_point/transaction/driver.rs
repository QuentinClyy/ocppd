use super::Transaction;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::messages::core::transaction::{AuthorizeRequest, Reason};
use crate::wire::ci_string::CiString;
use serde_json::json;

impl ChargePoint {
    pub(crate) fn intent(&mut self, out: &mut Vec<Output>, now: u64, intent: Intent) {
        if let Intent::Authorize { connector, id_tag } = &intent {
            return self.authorize(out, now, *connector, id_tag);
        }
        if intent == Intent::PowerCut {
            return self.power_cut(out, now);
        }
        if let Intent::FailTransfer { stage } = intent {
            return self.failing.add(stage);
        }
        let connector = intent.connector();
        let Some(index) = connector.and_then(|connector| self.index_of(connector)) else {
            return self.reject(out, now, connector, "unknown connector");
        };
        match intent {
            Intent::PlugIn { ev, .. } => self.plug_in(out, now, index, ev),
            Intent::Stop { .. } => self.stop_by_driver(out, now, index),
            Intent::Unplug { .. } => self.unplug(out, now, index),
            Intent::Fault { .. } => self.fault(out, now, index),
            Intent::ClearFault { .. } => self.station.connectors[index].repair(),
            Intent::LockFailure { .. } => self.station.connectors[index].lock_broken = true,
            Intent::Authorize { .. } | Intent::PowerCut | Intent::FailTransfer { .. } => {}
        }
    }

    fn plug_in(&mut self, out: &mut Vec<Output>, now: u64, index: usize, ev: EvSpec) {
        let connector = &mut self.station.connectors[index];
        if connector.plugged.is_some() {
            return self.reject(out, now, Some(index as u32 + 1), "already plugged in");
        }
        connector.plug_in(ev.ev(), ev.soc);
        self.start_held(out, now);
    }

    fn authorize(&mut self, out: &mut Vec<Output>, now: u64, connector: Option<u32>, id_tag: &str) {
        let id_tag = match CiString::new(id_tag) {
            Ok(id_tag) => id_tag,
            Err(err) => return self.reject(out, now, connector, &err.to_string()),
        };
        if self.resetting.is_some() {
            return self.reject(out, now, connector, "resetting");
        }
        if !self.is_accepted() && self.locally_authorized(&id_tag, now).is_none() {
            let reason = match self.is_connected() {
                true => "not registered",
                false => "offline",
            };
            return self.reject(out, now, connector, reason);
        }
        let index = match connector {
            Some(connector) => match self.index_of(connector) {
                Some(index) => index,
                None => return self.reject(out, now, Some(connector), "unknown connector"),
            },
            None => {
                let waiting: Vec<usize> = (0..self.transactions.len())
                    .filter(|&index| {
                        self.station.connectors[index].plugged.is_some()
                            && self.transactions[index] == Transaction::Idle
                            && !self.unavailable(index)
                    })
                    .collect();
                match waiting[..] {
                    [index] => index,
                    [] if self.held.is_some() => {
                        return self.reject(out, now, None, "authorization pending");
                    }
                    [] => return self.hold_for_authorization(out, now, id_tag, None),
                    _ => return self.reject(out, now, None, "ambiguous connector"),
                }
            }
        };
        let connector = Some(index as u32 + 1);
        if self.transactions[index] != Transaction::Idle {
            return self.reject(out, now, connector, "transaction in progress");
        }
        if self.unavailable(index) {
            return self.reject(out, now, connector, "unavailable");
        }
        let plugged = self.station.connectors[index].plugged.is_some();
        if self.held_for(index) || (!plugged && self.held.is_some()) {
            return self.reject(out, now, connector, "authorization pending");
        }
        if !plugged {
            return self.hold_for_authorization(out, now, id_tag, Some(index));
        }
        if let Some(info) = self.locally_authorized(&id_tag, now) {
            return self.authorized(out, now, index, id_tag, info);
        }
        let request = AuthorizeRequest {
            id_tag: id_tag.clone(),
        };
        self.outbox
            .push("Authorize", index as u32 + 1, json!(request));
        self.transactions[index] = Transaction::Authorizing { id_tag };
    }

    fn stop_by_driver(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        if !matches!(self.transactions[index], Transaction::Active { .. }) {
            return self.reject(out, now, Some(index as u32 + 1), "no active transaction");
        }
        self.stop(out, now, index, Reason::Local);
    }

    fn unplug(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        if self.station.connectors[index].plugged.is_none() {
            return self.reject(out, now, Some(index as u32 + 1), "nothing plugged in");
        }
        self.stop(out, now, index, Reason::EvDisconnected);
        self.station.connectors[index].unplug(now);
        if self.transactions[index] == Transaction::Finished {
            self.transactions[index] = Transaction::Idle;
        }
    }
}

#[cfg(test)]
#[path = "driver.test.rs"]
mod tests;
