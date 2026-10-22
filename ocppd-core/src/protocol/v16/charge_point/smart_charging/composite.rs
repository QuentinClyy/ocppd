use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::datatypes::charging_profile::{
    ChargingProfilePurpose, ChargingRateUnit, ChargingSchedule, ChargingSchedulePeriod,
};
use crate::protocol::v16::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v16::messages::smart_charging::{
    GetCompositeScheduleRequest, GetCompositeScheduleResponse,
};
use crate::smart_charging::composite;
use crate::smart_charging::schedule::Unit;
use crate::wire::date_time::DateTime;

impl ChargePoint {
    pub(crate) fn composite_limit_w(&self, connector: u32, at: u64) -> f64 {
        let started = self.transaction_start(connector);
        let purpose_limit = |connector, purpose| {
            let profiles = self.charging_profiles.stacked(connector, purpose);
            profiles
                .into_iter()
                .find_map(|schedule| schedule.limit_w(at, started, &self.station))
        };
        let mut limit = self.station.limit_w();
        if let Some(max) = purpose_limit(0, ChargingProfilePurpose::ChargePointMaxProfile) {
            limit = limit.min(max);
        }
        if started.is_some() {
            let transaction = purpose_limit(connector, ChargingProfilePurpose::TxProfile)
                .or_else(|| purpose_limit(connector, ChargingProfilePurpose::TxDefaultProfile))
                .or_else(|| purpose_limit(0, ChargingProfilePurpose::TxDefaultProfile));
            if let Some(transaction) = transaction {
                limit = limit.min(transaction);
            }
        }
        limit
    }

    pub(crate) fn apply_limits(&mut self, out: &mut Vec<Output>, now: u64) {
        let mut next_change: Option<u64> = None;
        for index in 0..self.transactions.len() {
            let connector = index as u32 + 1;
            if self.transaction_start(connector).is_none() {
                continue;
            }
            let limit_w = self.composite_limit_w(connector, now);
            let charging = &mut self.station.connectors[index];
            if charging
                .offered_w()
                .is_some_and(|offered| offered != limit_w)
            {
                charging.limit_to(now, limit_w);
                let kind = EventKind::ChargingLimitChanged { connector, limit_w };
                self.emit(out, now, kind);
            }
            let change = self.next_limit_change(connector, now);
            next_change = [next_change, change].into_iter().flatten().min();
        }
        self.limit_change_at = next_change;
    }

    pub(crate) fn next_limit_change(&self, connector: u32, at: u64) -> Option<u64> {
        let started = self.transaction_start(connector);
        self.charging_profiles
            .for_connector(connector)
            .filter_map(|schedule| schedule.next_change(at, started))
            .min()
    }

    pub(crate) fn composite_schedule(
        &self,
        now: u64,
        request: GetCompositeScheduleRequest,
    ) -> GetCompositeScheduleResponse {
        let connector = request.connector_id;
        if connector != 0 && self.index_of(connector).is_none() {
            return GetCompositeScheduleResponse {
                status: GenericStatus::Rejected,
                connector_id: None,
                schedule_start: None,
                charging_schedule: None,
            };
        }
        let unit = request.charging_rate_unit.unwrap_or(ChargingRateUnit::W);
        let end = now + u64::from(request.duration) * 1000;
        let sampling = match unit {
            ChargingRateUnit::W => Unit::Watts,
            ChargingRateUnit::A => Unit::Amps,
        };
        let periods = composite::periods(
            &self.station,
            sampling,
            now,
            end,
            |at| self.composite_limit_w(connector, at),
            |at| self.next_limit_change(connector, at),
        );
        let periods = periods.into_iter().map(|period| ChargingSchedulePeriod {
            start_period: period.start_s,
            limit: period.limit,
            number_phases: None,
        });
        GetCompositeScheduleResponse {
            status: GenericStatus::Accepted,
            connector_id: Some(connector),
            schedule_start: Some(DateTime { unix_ms: now }),
            charging_schedule: Some(ChargingSchedule {
                duration: Some(request.duration),
                start_schedule: Some(DateTime { unix_ms: now }),
                charging_rate_unit: unit,
                charging_schedule_period: periods.collect(),
                min_charging_rate: None,
            }),
        }
    }

    fn transaction_start(&self, connector: u32) -> Option<u64> {
        let index = self.index_of(connector)?;
        match self.transactions[index] {
            Transaction::Active { start, .. } => Some(start),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "composite.test.rs"]
mod tests;
