use super::Reservation;
use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::ReservationEnd;
use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::reservation::ConnectorType;
use crate::wire::ci_string::CiString;

pub const CONNECTOR_TYPE: ConnectorType = ConnectorType::CType2;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn reserved(&self, index: usize) -> bool {
        let floating = self.floating(None);
        self.reservations
            .iter()
            .any(|reservation| reservation.evse == Some(index))
            || (self.is_free(index, None) && floating > 0 && self.free(None, None) <= floating)
    }

    pub(in crate::protocol::v201::charging_station) fn reservation_allows(
        &self,
        index: usize,
        id_token: &IdToken,
        group: Option<&CiString<36>>,
    ) -> bool {
        if let Some(reservation) = self.specific(index) {
            return reservation.admits(id_token, group);
        }
        let mut reservations = self.reservations.iter();
        reservations.any(|reservation| reservation.claimable(index, id_token, group))
            || self.free(Some(index), None) >= self.floating(None)
    }

    pub(in crate::protocol::v201::charging_station) fn claim_reservation(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        id_token: &IdToken,
        group: Option<&CiString<36>>,
    ) -> bool {
        if !self.reservation_allows(index, id_token, group) {
            return false;
        }
        let position = self.reservations.iter().position(|reservation| {
            reservation.evse == Some(index) || reservation.claimable(index, id_token, group)
        });
        if let Some(position) = position {
            self.sessions[index].reservation_id = Some(self.reservations[position].id);
            self.end_reservation(out, now, position, ReservationEnd::Used);
        }
        true
    }

    pub(super) fn has_room_for(&self, reservation: &Reservation) -> bool {
        let except = Some(reservation.id);
        let floating = self.floating(except);
        match reservation.evse {
            Some(index) => {
                self.is_free(index, except) && self.free(Some(index), except) >= floating
            }
            None => self.free(None, except) > floating,
        }
    }

    fn specific(&self, index: usize) -> Option<&Reservation> {
        let mut reservations = self.reservations.iter();
        reservations.find(|reservation| reservation.evse == Some(index))
    }

    fn is_free(&self, index: usize, except: Option<i32>) -> bool {
        let reserved = self
            .reservations
            .iter()
            .any(|reservation| Some(reservation.id) != except && reservation.evse == Some(index));
        self.operative(index)
            && self.station.connectors[index].plugged.is_none()
            && self.sessions[index].transaction.is_none()
            && !reserved
    }

    fn free(&self, besides: Option<usize>, except: Option<i32>) -> usize {
        let evses = 0..self.sessions.len();
        evses
            .filter(|&index| Some(index) != besides && self.is_free(index, except))
            .count()
    }

    fn floating(&self, except: Option<i32>) -> usize {
        let reservations = self.reservations.iter();
        reservations
            .filter(|reservation| reservation.evse.is_none() && Some(reservation.id) != except)
            .count()
    }
}

impl Reservation {
    pub(super) fn fits(&self, index: usize) -> bool {
        self.evse.is_none_or(|evse| evse == index)
            && self
                .connector_type
                .is_none_or(|kind| kind == CONNECTOR_TYPE)
    }

    fn claimable(&self, index: usize, id_token: &IdToken, group: Option<&CiString<36>>) -> bool {
        self.fits(index) && self.admits(id_token, group)
    }

    fn admits(&self, id_token: &IdToken, group: Option<&CiString<36>>) -> bool {
        let same_group = match (&self.group, group) {
            (Some(reserved), Some(group)) => reserved.matches(group),
            _ => false,
        };
        self.id_token.id_token.matches(&id_token.id_token) || same_group
    }
}

#[cfg(test)]
#[path = "pool.test.rs"]
mod tests;
