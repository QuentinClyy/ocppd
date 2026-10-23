use crate::protocol::shared::smart_charging::composite;
use crate::protocol::shared::smart_charging::schedule::Unit;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::messages::datatypes::charging_profile::{
    ChargingRateUnit, ChargingSchedulePeriod,
};
use crate::protocol::v201::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v201::messages::smart_charging::composite::{
    CompositeSchedule, GetCompositeScheduleRequest, GetCompositeScheduleResponse,
};
use crate::wire::date_time::DateTime;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn composite_schedule(
        &self,
        now: u64,
        request: GetCompositeScheduleRequest,
    ) -> GetCompositeScheduleResponse {
        let evse = request.evse_id;
        if evse as usize > self.sessions.len() {
            return GetCompositeScheduleResponse {
                status: GenericStatus::Rejected,
                status_info: None,
                schedule: None,
            };
        }
        let unit = request.charging_rate_unit.unwrap_or(ChargingRateUnit::W);
        let sampling = match unit {
            ChargingRateUnit::W => Unit::Watts,
            ChargingRateUnit::A => Unit::Amps,
        };
        let end = now + u64::from(request.duration) * 1000;
        let periods = match evse.checked_sub(1) {
            None => composite::periods(
                &self.station,
                sampling,
                now,
                end,
                |at| self.station_limit_w(at),
                |at| self.next_limit_change(0, at, None),
            ),
            Some(index) => {
                let index = index as usize;
                let started = Some(self.transaction_start(index).unwrap_or(now));
                composite::periods(
                    &self.station,
                    sampling,
                    now,
                    end,
                    |at| self.limit_from(index, at, started),
                    |at| self.next_limit_change(evse, at, started),
                )
            }
        };
        let periods = periods.into_iter().map(|period| ChargingSchedulePeriod {
            start_period: period.start_s,
            limit: period.limit,
            number_phases: None,
            phase_to_use: None,
        });
        GetCompositeScheduleResponse {
            status: GenericStatus::Accepted,
            status_info: None,
            schedule: Some(CompositeSchedule {
                charging_schedule_period: periods.collect(),
                evse_id: evse,
                duration: request.duration,
                schedule_start: DateTime { unix_ms: now },
                charging_rate_unit: unit,
            }),
        }
    }
}

#[cfg(test)]
#[path = "composite.test.rs"]
mod tests;
