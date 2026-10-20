use super::ChargePoint;
use super::schedule;
use super::transaction::Transaction;
use crate::v16::messages::datatypes::charging_profile::{
    ChargingProfilePurpose, ChargingRateUnit, ChargingSchedule, ChargingSchedulePeriod,
};
use crate::v16::messages::datatypes::generic_status::GenericStatus;
use crate::v16::messages::smart_charging::{
    GetCompositeScheduleRequest, GetCompositeScheduleResponse,
};
use crate::wire::date_time::DateTime;

impl ChargePoint {
    pub(super) fn composite_limit_w(&self, connector: u32, at: u64) -> f64 {
        let started = self.transaction_start(connector);
        let purpose_limit = |connector, purpose| {
            let profiles = self.charging_profiles.stacked(connector, purpose);
            profiles
                .into_iter()
                .find_map(|profile| schedule::limit_w(profile, at, started, &self.station))
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

    pub(super) fn next_limit_change(&self, connector: u32, at: u64) -> Option<u64> {
        let started = self.transaction_start(connector);
        self.charging_profiles
            .for_connector(connector)
            .filter_map(|profile| schedule::next_change(profile, at, started))
            .min()
    }

    pub(super) fn composite_schedule(
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
        let mut periods: Vec<ChargingSchedulePeriod> = Vec::new();
        let mut at = now;
        while at < end {
            let limit = self.in_unit(self.composite_limit_w(connector, at), unit);
            if periods.last().is_none_or(|last| last.limit != limit) {
                periods.push(ChargingSchedulePeriod {
                    start_period: ((at - now) / 1000) as u32,
                    limit,
                    number_phases: None,
                });
            }
            at = self.next_limit_change(connector, at).unwrap_or(end);
        }
        GetCompositeScheduleResponse {
            status: GenericStatus::Accepted,
            connector_id: Some(connector),
            schedule_start: Some(DateTime { unix_ms: now }),
            charging_schedule: Some(ChargingSchedule {
                duration: Some(request.duration),
                start_schedule: Some(DateTime { unix_ms: now }),
                charging_rate_unit: unit,
                charging_schedule_period: periods,
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

    fn in_unit(&self, watts: f64, unit: ChargingRateUnit) -> f64 {
        match unit {
            ChargingRateUnit::W => watts.round(),
            ChargingRateUnit::A => {
                let amps = watts / (self.station.voltage_v * f64::from(self.station.phases));
                (amps * 10.0).round() / 10.0
            }
        }
    }
}

#[cfg(test)]
#[path = "composite.test.rs"]
mod tests;
