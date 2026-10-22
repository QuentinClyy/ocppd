use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::charging_profile::ChargingProfilePurpose;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn limit_w(
        &self,
        index: usize,
        at: u64,
    ) -> f64 {
        self.limit_from(index, at, self.transaction_start(index))
    }

    pub(super) fn limit_from(&self, index: usize, at: u64, started: Option<u64>) -> f64 {
        let evse = index as u32 + 1;
        let first = |evse, purpose| self.first_limit(evse, purpose, at, started);
        let mut limit = self.station.limit_w();
        if let Some(max) = first(0, ChargingProfilePurpose::ChargingStationMaxProfile) {
            limit = limit.min(max / self.sharing_evses() as f64);
        }
        if started.is_some() {
            let transaction = first(evse, ChargingProfilePurpose::TxProfile)
                .or_else(|| first(evse, ChargingProfilePurpose::TxDefaultProfile))
                .or_else(|| first(0, ChargingProfilePurpose::TxDefaultProfile));
            if let Some(transaction) = transaction {
                limit = limit.min(transaction);
            }
        }
        limit
    }

    pub(super) fn station_limit_w(&self, at: u64) -> f64 {
        let hardware = self.station.limit_w() * self.sessions.len() as f64;
        let max = self.first_limit(
            0,
            ChargingProfilePurpose::ChargingStationMaxProfile,
            at,
            None,
        );
        max.map_or(hardware, |max| hardware.min(max))
    }

    pub(super) fn next_limit_change(
        &self,
        evse: u32,
        at: u64,
        started: Option<u64>,
    ) -> Option<u64> {
        self.charging_profiles
            .for_evse(evse)
            .filter_map(|schedule| schedule.next_change(at, started))
            .min()
    }

    pub(super) fn transaction_start(&self, index: usize) -> Option<u64> {
        let transaction = self.sessions[index].transaction.as_ref()?;
        Some(transaction.started)
    }

    pub(in crate::protocol::v201::charging_station) fn apply_limits(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
    ) {
        let mut next_change: Option<u64> = None;
        for index in 0..self.sessions.len() {
            let Some(offered) = self.station.connectors[index].offered_w() else {
                continue;
            };
            let limit_w = self.limit_w(index, now);
            let evse = index as u32 + 1;
            if offered != limit_w {
                self.station.connectors[index].limit_to(now, limit_w);
                self.emit(out, now, EventKind::ChargingLimitChanged { evse, limit_w });
            }
            let change = self.next_limit_change(evse, now, self.transaction_start(index));
            next_change = next_change.into_iter().chain(change).min();
        }
        self.limit_change_at = next_change;
    }

    fn first_limit(
        &self,
        evse: u32,
        purpose: ChargingProfilePurpose,
        at: u64,
        started: Option<u64>,
    ) -> Option<f64> {
        let schedules = self.charging_profiles.stacked(evse, purpose);
        schedules
            .into_iter()
            .find_map(|schedule| schedule.limit_w(at, started, &self.station))
    }

    fn sharing_evses(&self) -> usize {
        let charging = self.station.connectors.iter();
        let charging = charging.filter(|connector| connector.offered_w().is_some());
        charging.count().max(1)
    }
}

#[cfg(test)]
#[path = "limit.test.rs"]
mod tests;
