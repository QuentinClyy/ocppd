use super::ChargePoint;
use super::transaction::Transaction;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Output;
use crate::v16::messages::core::transaction::{AuthorizeRequest, Reason};
use crate::wire::ci_string::CiString;
use serde_json::json;

impl ChargePoint {
    pub(super) fn intent(&mut self, out: &mut Vec<Output>, now: u64, intent: Intent) {
        if let Intent::Authorize { connector, id_tag } = &intent {
            return self.authorize(out, now, *connector, id_tag);
        }
        let connector = intent.connector();
        let Some(index) = connector.and_then(|connector| self.index_of(connector)) else {
            return self.reject(out, now, connector, "unknown connector");
        };
        match intent {
            Intent::PlugIn { ev, .. } => self.plug_in(out, now, index, ev),
            Intent::Stop { .. } => self.stop_by_driver(out, now, index),
            Intent::Unplug { .. } => self.unplug(out, now, index),
            Intent::Authorize { .. } => {}
        }
    }

    fn plug_in(&mut self, out: &mut Vec<Output>, now: u64, index: usize, ev: EvSpec) {
        let connector = &mut self.station.connectors[index];
        if connector.plugged.is_some() {
            return self.reject(out, now, Some(index as u32 + 1), "already plugged in");
        }
        connector.plug_in(ev.ev(), ev.soc);
        self.start_held(now);
    }

    fn authorize(&mut self, out: &mut Vec<Output>, now: u64, connector: Option<u32>, id_tag: &str) {
        let id_tag = match CiString::new(id_tag) {
            Ok(id_tag) => id_tag,
            Err(err) => return self.reject(out, now, connector, &err.to_string()),
        };
        if self.held.is_some() {
            return self.reject(out, now, connector, "authorization pending");
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
                    [] => return self.hold_for_authorization(id_tag, None),
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
        if self.station.connectors[index].plugged.is_none() {
            return self.hold_for_authorization(id_tag, Some(index));
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
        match self.transactions[index] {
            Transaction::Active { .. } => self.stop(out, now, index, Reason::EvDisconnected),
            Transaction::Finished => self.transactions[index] = Transaction::Idle,
            _ => {}
        }
        self.station.connectors[index].unplug(now);
    }
}

#[cfg(test)]
#[path = "driver.test.rs"]
mod tests;
