mod fault;
mod status;

use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::availability::{
    ChangeAvailabilityRequest, ChangeAvailabilityResponse, ChangeAvailabilityStatus,
    OperationalStatus,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Availability {
    pub operative: bool,
    pub scheduled: Option<bool>,
}

impl Default for Availability {
    fn default() -> Self {
        Availability {
            operative: true,
            scheduled: None,
        }
    }
}

impl ChargingStation {
    pub(super) fn change_availability(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ChangeAvailabilityRequest,
    ) -> ChangeAvailabilityResponse {
        let operative = request.operational_status == OperationalStatus::Operative;
        let target = match request.evse {
            None => None,
            Some(evse) if evse.id == 0 && evse.connector_id.is_none() => None,
            Some(evse) => {
                let index = evse.id.checked_sub(1).map(|index| index as usize);
                let known = index.is_some_and(|index| index < self.evses.len())
                    && evse.connector_id.is_none_or(|connector| connector == 1);
                match (known, index) {
                    (true, Some(index)) => Some(index),
                    _ => return response(ChangeAvailabilityStatus::Rejected),
                }
            }
        };
        let current = match target {
            Some(index) => self.evses[index],
            None => self.station_availability,
        };
        if current.operative == operative {
            if let Some(index) = target {
                self.evses[index].scheduled = None;
            } else {
                self.station_availability.scheduled = None;
                self.resend_statuses = true;
            }
            return response(ChangeAvailabilityStatus::Accepted);
        }
        let busy = match target {
            Some(index) => self.sessions[index].transaction.is_some(),
            None => self
                .sessions
                .iter()
                .any(|session| session.transaction.is_some()),
        };
        let slot = match target {
            Some(index) => &mut self.evses[index],
            None => &mut self.station_availability,
        };
        if busy {
            slot.scheduled = Some(operative);
            return response(ChangeAvailabilityStatus::Scheduled);
        }
        slot.operative = operative;
        let evse = target.map(|index| index as u32 + 1);
        self.emit(out, now, EventKind::AvailabilityChanged { evse, operative });
        response(ChangeAvailabilityStatus::Accepted)
    }

    pub(super) fn apply_scheduled_availability(&mut self, out: &mut Vec<Output>, now: u64) {
        let idle =
            |index: usize, station: &ChargingStation| station.sessions[index].transaction.is_none();
        for index in 0..self.evses.len() {
            if let Some(operative) = self.evses[index].scheduled.filter(|_| idle(index, self)) {
                self.evses[index] = Availability {
                    operative,
                    scheduled: None,
                };
                let evse = Some(index as u32 + 1);
                self.emit(out, now, EventKind::AvailabilityChanged { evse, operative });
            }
        }
        let all_idle = (0..self.evses.len()).all(|index| idle(index, self));
        if let Some(operative) = self.station_availability.scheduled.filter(|_| all_idle) {
            self.station_availability = Availability {
                operative,
                scheduled: None,
            };
            self.emit(
                out,
                now,
                EventKind::AvailabilityChanged {
                    evse: None,
                    operative,
                },
            );
        }
    }

    pub(super) fn operative(&self, index: usize) -> bool {
        self.station_availability.operative
            && self.evses[index].operative
            && self.station.connectors[index].fault.is_none()
    }
}

fn response(status: ChangeAvailabilityStatus) -> ChangeAvailabilityResponse {
    ChangeAvailabilityResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
