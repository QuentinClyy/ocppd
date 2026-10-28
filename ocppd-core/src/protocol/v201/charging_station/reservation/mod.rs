mod pool;

use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::event::{EventKind, ReservationEnd};
use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::reservation::{
    CancelReservationRequest, CancelReservationResponse, CancelReservationStatus, ConnectorType,
    ReservationStatusUpdateRequest, ReservationUpdateStatus, ReserveNowRequest, ReserveNowResponse,
    ReserveNowStatus,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

pub const RESERVATION_CTRLR: &str = "ReservationCtrlr";
pub const NON_EVSE_SPECIFIC: &str = "NonEvseSpecific";

pub struct Reservation {
    id: i32,
    evse: Option<usize>,
    connector_type: Option<ConnectorType>,
    expires: u64,
    id_token: IdToken,
    group: Option<CiString<36>>,
}

impl ChargingStation {
    pub(super) fn reserve_now(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ReserveNowRequest,
    ) -> ReserveNowResponse {
        let setting = |name| self.device_model.value(RESERVATION_CTRLR, name) == Some("true");
        let evse = match request.evse_id {
            None if setting(NON_EVSE_SPECIFIC) => None,
            None => return response(ReserveNowStatus::Rejected),
            Some(evse) => match self.evse_index(evse) {
                Some(index) => Some(index),
                None => return response(ReserveNowStatus::Rejected),
            },
        };
        let reservation = Reservation {
            id: request.id,
            evse,
            connector_type: request.connector_type,
            expires: request.expiry_date_time.unix_ms,
            id_token: request.id_token,
            group: request.group_id_token.map(|group| group.id_token),
        };
        let evses = 0..self.sessions.len();
        let connectors = &self.station.connectors;
        let fits = |&index: &usize| reservation.fits(index, connectors[index].current);
        let targets: Vec<usize> = evses.filter(fits).collect();
        if !setting("Enabled") || targets.is_empty() {
            return response(ReserveNowStatus::Rejected);
        }
        let connectors = &self.station.connectors;
        if targets
            .iter()
            .all(|&index| connectors[index].fault.is_some())
        {
            return response(ReserveNowStatus::Faulted);
        }
        if !targets.iter().any(|&index| self.operative(index)) {
            return response(ReserveNowStatus::Unavailable);
        }
        if !self.has_room_for(&reservation) {
            return response(ReserveNowStatus::Occupied);
        }
        let kind = EventKind::Reserved {
            evse: evse.map(|index| index as u32 + 1),
            reservation_id: reservation.id,
            id_tag: reservation.id_token.id_token.as_str().to_owned(),
            expires: reservation.expires,
        };
        self.reservations.retain(|kept| kept.id != reservation.id);
        self.reservations.push(reservation);
        self.emit(out, now, kind);
        response(ReserveNowStatus::Accepted)
    }

    pub(super) fn cancel_reservation(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: CancelReservationRequest,
    ) -> CancelReservationResponse {
        let position = self
            .reservations
            .iter()
            .position(|reservation| reservation.id == request.reservation_id);
        let status = match position {
            Some(position) => {
                self.end_reservation(out, now, position, ReservationEnd::Cancelled);
                CancelReservationStatus::Accepted
            }
            None => CancelReservationStatus::Rejected,
        };
        CancelReservationResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn reservation_deadline(&self) -> Option<u64> {
        let expiries = self
            .reservations
            .iter()
            .map(|reservation| reservation.expires);
        expiries.min()
    }

    pub(super) fn reservations_tick(&mut self, out: &mut Vec<Output>, now: u64) {
        let evses = 0..self.sessions.len();
        while let Some(position) = self.reservations.iter().position(|reservation| {
            let connectors = &self.station.connectors;
            let fits = |&index: &usize| reservation.fits(index, connectors[index].current);
            let mut targets = evses.clone().filter(fits);
            now >= reservation.expires || !targets.any(|index| self.operative(index))
        }) {
            let reservation = &self.reservations[position];
            let (end, update) = match now >= reservation.expires {
                true => (ReservationEnd::Expired, ReservationUpdateStatus::Expired),
                false => (ReservationEnd::Removed, ReservationUpdateStatus::Removed),
            };
            let request = ReservationStatusUpdateRequest {
                reservation_id: reservation.id,
                reservation_update_status: update,
            };
            self.outbox
                .push("ReservationStatusUpdate", 0, json!(request));
            self.end_reservation(out, now, position, end);
        }
    }

    fn end_reservation(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        position: usize,
        reason: ReservationEnd,
    ) {
        let reservation = self.reservations.remove(position);
        let kind = EventKind::ReservationEnded {
            evse: reservation.evse.map(|index| index as u32 + 1),
            reservation_id: reservation.id,
            reason,
        };
        self.emit(out, now, kind);
    }

    fn evse_index(&self, evse: u32) -> Option<usize> {
        let index = evse.checked_sub(1)? as usize;
        (index < self.sessions.len()).then_some(index)
    }
}

fn response(status: ReserveNowStatus) -> ReserveNowResponse {
    ReserveNowResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
