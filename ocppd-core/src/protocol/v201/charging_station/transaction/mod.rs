pub(super) mod authorize;
pub(super) mod event;
mod points;
mod reliability;
mod sampling;

use super::ChargingStation;
use super::device_model::table::{TX_CTRLR, TX_START_POINT, TX_STOP_POINT};
use super::tariff_cost::TOTAL_COST_FALLBACK_MESSAGE;
use crate::api::io::Output;
use crate::protocol::v201::messages::datatypes::charging_profile::ChargingProfile;
use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::datatypes::meter_value::MeterValue;
use crate::protocol::v201::messages::transactions::{
    ChargingState, StoppedReason, TransactionEventType, TriggerReason,
};
use crate::wire::ci_string::CiString;
use points::{TxPoint, stopped_reason};

#[derive(Default)]
pub struct Session {
    pub authorization: Authorization,
    pub transaction: Option<Transaction>,
    pub checking: Option<IdToken>,
    pub stopped_by: Option<IdToken>,
    pub suspended: bool,
    pub remote_start_id: Option<i32>,
    pub remote_profile: Option<ChargingProfile>,
    pub reservation_id: Option<i32>,
}

#[derive(Default, Clone, PartialEq)]
pub enum Authorization {
    #[default]
    None,
    Pending(IdToken),
    Authorized {
        id_token: IdToken,
        group: Option<CiString<36>>,
        at: u64,
    },
}

pub struct Transaction {
    pub id: CiString<36>,
    pub seq_no: u32,
    pub charging_state: ChargingState,
    pub id_token_sent: bool,
    pub stop_reason: Option<StoppedReason>,
    pub started: u64,
    reached: Vec<TxPoint>,
    last_periodic: u64,
    last_ended_sample: u64,
    last_clock_sample: u64,
    ended_samples: Vec<MeterValue>,
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
        let valid = self.valid_points(now, index);
        let state = self.charging_state(now, index);
        let Some(transaction) = &mut self.sessions[index].transaction else {
            let start_points = self.points(TX_START_POINT);
            let mut reached = start_points.iter().filter(|point| valid.contains(point));
            let starts = reached.clone().next().is_some() && self.operative(index);
            let on_energy = reached.all(|point| *point == TxPoint::EnergyTransfer);
            if starts {
                self.sessions[index].transaction = Some(Transaction {
                    id: transaction_id(now, index),
                    seq_no: 0,
                    charging_state: ChargingState::Idle,
                    id_token_sent: false,
                    stop_reason: None,
                    started: now,
                    reached: valid,
                    last_periodic: now,
                    last_ended_sample: now,
                    last_clock_sample: now,
                    ended_samples: self.ended_sample(now, index, false),
                });
                self.attach_remote_profile(out, now, index);
                let begin = self.started_sample(now, index);
                let started = TransactionEventType::Started;
                if !on_energy {
                    return self.send_transaction_event(out, now, index, started, trigger, begin);
                }
                let trigger = match trigger {
                    TriggerReason::CablePluggedIn => TriggerReason::ChargingStateChanged,
                    other => other,
                };
                self.push_transaction_event(out, now, index, (started, trigger, state), begin);
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
            let reason = stopped_reason(trigger, state);
            return self.end_transaction(out, now, index, trigger, reason);
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

    pub(super) fn end_transaction(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        trigger: TriggerReason,
        reason: StoppedReason,
    ) {
        let Some(transaction) = &mut self.sessions[index].transaction else {
            return;
        };
        transaction.stop_reason = Some(reason);
        self.sessions[index].authorization = Authorization::None;
        self.apply_charging(now, index);
        let samples = self.ended_samples(now, index);
        let ended = TransactionEventType::Ended;
        self.send_transaction_event(out, now, index, ended, trigger, samples);
        self.evaluate_monitors(out, now);
        if let Some(transaction) = self.sessions[index].transaction.take() {
            self.drop_transaction_messages(out, now, &transaction.id);
        }
        if !self.is_accepted() {
            self.show_fallback(out, now, index, TOTAL_COST_FALLBACK_MESSAGE);
        }
        self.sessions[index] = Session::default();
        if self.charging_profiles.end_transaction(index as u32 + 1) {
            self.count_profiles();
        }
        self.apply_charging(now, index);
        self.apply_scheduled_availability(out, now);
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
            (true, true) if self.sessions[index].suspended => ChargingState::SuspendedEvse,
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
        let suspended = self.sessions[index].suspended || !self.operative(index);
        let limit_w = self.limit_w(index, now);
        let connector = &mut self.station.connectors[index];
        let plugged = connector.plugged.is_some();
        let charging = connector
            .plugged
            .as_ref()
            .is_some_and(|plugged| plugged.charging.is_some());
        let allowed = plugged && authorized && !suspended;
        match (allowed, charging) {
            (true, false) => connector.start_charging(now, limit_w),
            (false, true) => connector.stop_charging(now),
            _ => {}
        }
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
