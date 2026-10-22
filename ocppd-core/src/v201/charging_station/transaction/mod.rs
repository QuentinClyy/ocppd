pub(super) mod authorize;
pub(super) mod event;
mod sampling;

use super::ChargingStation;
use super::device_model::table::{TX_CTRLR, TX_START_POINT, TX_STOP_POINT};
use crate::api::io::Output;
use crate::v201::messages::datatypes::id_token::IdToken;
use crate::v201::messages::datatypes::meter_value::MeterValue;
use crate::v201::messages::transactions::{ChargingState, TransactionEventType, TriggerReason};
use crate::wire::ci_string::CiString;

#[derive(Default)]
pub struct Session {
    pub authorization: Authorization,
    pub transaction: Option<Transaction>,
}

#[derive(Default, Clone, PartialEq)]
pub enum Authorization {
    #[default]
    None,
    Pending(IdToken),
    Authorized {
        id_token: IdToken,
        at: u64,
    },
}

pub struct Transaction {
    pub id: CiString<36>,
    pub seq_no: u32,
    pub charging_state: ChargingState,
    pub id_token_sent: bool,
    reached: Vec<TxPoint>,
    last_periodic: u64,
    last_ended_sample: u64,
    last_clock_sample: u64,
    ended_samples: Vec<MeterValue>,
}

#[derive(Clone, Copy, PartialEq)]
enum TxPoint {
    EvConnected,
    Authorized,
    PowerPathClosed,
    EnergyTransfer,
}

impl ChargingStation {
    pub(super) fn settle(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        trigger: TriggerReason,
    ) {
        if self.is_accepted() {
            self.report_statuses(out, now);
        }
        self.apply_charging(now, index);
        let valid = self.valid_points(index);
        let state = self.charging_state(now, index);
        let Some(transaction) = &mut self.sessions[index].transaction else {
            let starts = self
                .points(TX_START_POINT)
                .iter()
                .any(|point| valid.contains(point));
            if starts {
                self.sessions[index].transaction = Some(Transaction {
                    id: transaction_id(now, index),
                    seq_no: 0,
                    charging_state: state,
                    id_token_sent: false,
                    reached: valid,
                    last_periodic: now,
                    last_ended_sample: now,
                    last_clock_sample: now,
                    ended_samples: self.ended_sample(now, index, false),
                });
                let begin = self.started_sample(now, index);
                let started = TransactionEventType::Started;
                self.send_transaction_event(out, now, index, started, trigger, begin);
            }
            return;
        };
        for point in &valid {
            if !transaction.reached.contains(point) {
                transaction.reached.push(*point);
            }
        }
        let reached = transaction.reached.clone();
        let lost = self
            .points(TX_STOP_POINT)
            .iter()
            .any(|point| reached.contains(point) && !valid.contains(point));
        if lost {
            let samples = self.ended_samples(now, index);
            let ended = TransactionEventType::Ended;
            self.send_transaction_event(out, now, index, ended, trigger, samples);
            self.sessions[index] = Session::default();
            self.apply_charging(now, index);
            return;
        }
        let authorized = matches!(
            self.sessions[index].authorization,
            Authorization::Authorized { .. }
        );
        let Some(transaction) = &self.sessions[index].transaction else {
            return;
        };
        if state != transaction.charging_state || (authorized && !transaction.id_token_sent) {
            let updated = TransactionEventType::Updated;
            self.send_transaction_event(out, now, index, updated, trigger, Vec::new());
        }
    }

    pub(super) fn charging_state(&self, now: u64, index: usize) -> ChargingState {
        let connector = &self.station.connectors[index];
        let authorized = matches!(
            self.sessions[index].authorization,
            Authorization::Authorized { .. }
        );
        match (connector.plugged.is_some(), authorized) {
            (false, _) => ChargingState::Idle,
            (true, false) => ChargingState::EvConnected,
            (true, true) if connector.finish_time().is_some_and(|at| at <= now) => {
                ChargingState::SuspendedEv
            }
            (true, true) => ChargingState::Charging,
        }
    }

    pub(super) fn transaction_deadline(&self) -> Option<u64> {
        (0..self.sessions.len())
            .filter_map(|index| {
                let session = &self.sessions[index];
                let connector = &self.station.connectors[index];
                let timeout = match session.authorization {
                    Authorization::Authorized { at, .. } if connector.plugged.is_none() => {
                        Some(at + self.connection_timeout_ms())
                    }
                    _ => None,
                };
                let charging = session
                    .transaction
                    .as_ref()
                    .filter(|transaction| transaction.charging_state == ChargingState::Charging);
                let full = charging.and_then(|_| connector.finish_time());
                timeout.into_iter().chain(full).min()
            })
            .min()
    }

    pub(super) fn transaction_tick(&mut self, out: &mut Vec<Output>, now: u64) {
        for index in 0..self.sessions.len() {
            let session = &self.sessions[index];
            let timed_out = match session.authorization {
                Authorization::Authorized { at, .. } => {
                    self.station.connectors[index].plugged.is_none()
                        && now >= at + self.connection_timeout_ms()
                }
                _ => false,
            };
            if timed_out {
                self.sessions[index].authorization = Authorization::None;
                self.settle(out, now, index, TriggerReason::EvConnectTimeout);
                continue;
            }
            let state = self.charging_state(now, index);
            let changed = session
                .transaction
                .as_ref()
                .is_some_and(|transaction| transaction.charging_state != state);
            if changed {
                self.settle(out, now, index, TriggerReason::ChargingStateChanged);
            }
            self.sample_transaction(out, now, index);
        }
    }

    fn apply_charging(&mut self, now: u64, index: usize) {
        let authorized = matches!(
            self.sessions[index].authorization,
            Authorization::Authorized { .. }
        );
        let limit_w = self.station.limit_w();
        let connector = &mut self.station.connectors[index];
        let plugged = connector.plugged.is_some();
        let charging = connector
            .plugged
            .as_ref()
            .is_some_and(|plugged| plugged.charging.is_some());
        match (plugged && authorized, charging) {
            (true, false) => connector.start_charging(now, limit_w),
            (false, true) => connector.stop_charging(now),
            _ => {}
        }
    }

    fn valid_points(&self, index: usize) -> Vec<TxPoint> {
        let plugged = self.station.connectors[index].plugged.is_some();
        let authorized = matches!(
            self.sessions[index].authorization,
            Authorization::Authorized { .. }
        );
        let energy = self.station.connectors[index]
            .plugged
            .as_ref()
            .is_some_and(|plugged| plugged.charging.is_some());
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

    fn points(&self, variable: &str) -> Vec<TxPoint> {
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

    fn connection_timeout_ms(&self) -> u64 {
        let seconds = self.device_model.number(TX_CTRLR, "EVConnectionTimeOut");
        seconds.unwrap_or(0) * 1000
    }
}

fn transaction_id(now: u64, index: usize) -> CiString<36> {
    let evse = index as u32 + 1;
    let id = format!("{now:x}-{evse}");
    CiString::new(&id).unwrap_or_else(|_| unreachable!("16 hex digits, a dash and a u32 fit in 36"))
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
