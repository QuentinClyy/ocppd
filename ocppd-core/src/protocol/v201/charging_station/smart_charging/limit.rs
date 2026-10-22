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
        let evse = index as u32 + 1;
        let started = self.transaction_start(index);
        let first = |evse, purpose| {
            let schedules = self.charging_profiles.stacked(evse, purpose);
            schedules
                .into_iter()
                .find_map(|schedule| schedule.limit_w(at, started, &self.station))
        };
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
            if offered != limit_w {
                self.station.connectors[index].limit_to(now, limit_w);
                let evse = index as u32 + 1;
                self.emit(out, now, EventKind::ChargingLimitChanged { evse, limit_w });
            }
            let started = self.transaction_start(index);
            let change = self
                .charging_profiles
                .for_evse(index as u32 + 1)
                .filter_map(|schedule| schedule.next_change(now, started))
                .min();
            next_change = next_change.into_iter().chain(change).min();
        }
        self.limit_change_at = next_change;
    }

    fn transaction_start(&self, index: usize) -> Option<u64> {
        let transaction = self.sessions[index].transaction.as_ref()?;
        Some(transaction.started)
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
