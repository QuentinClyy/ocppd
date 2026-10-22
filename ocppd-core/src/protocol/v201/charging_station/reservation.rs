use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::event::{EventKind, ReservationEnd};
use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::reservation::{
    CancelReservationRequest, CancelReservationResponse, CancelReservationStatus,
    ReservationStatusUpdateRequest, ReservationUpdateStatus, ReserveNowRequest, ReserveNowResponse,
    ReserveNowStatus,
};
use crate::wire::ci_string::CiString;
use serde_json::json;

pub const RESERVATION_CTRLR: &str = "ReservationCtrlr";

pub struct Reservation {
    id: i32,
    index: usize,
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
        let enabled = self.device_model.value(RESERVATION_CTRLR, "Enabled") == Some("true");
        let index = request
            .evse_id
            .and_then(|evse| evse.checked_sub(1))
            .map(|index| index as usize)
            .filter(|&index| index < self.sessions.len());
        let Some(index) = index.filter(|_| enabled) else {
            return response(ReserveNowStatus::Rejected);
        };
        if !self.operative(index) {
            return response(ReserveNowStatus::Unavailable);
        }
        let busy = self.station.connectors[index].plugged.is_some()
            || self.sessions[index].transaction.is_some()
            || self
                .reservations
                .iter()
                .any(|reservation| reservation.index == index && reservation.id != request.id);
        if busy {
            return response(ReserveNowStatus::Occupied);
        }
        let expires = request.expiry_date_time.unix_ms;
        let kind = EventKind::Reserved {
            evse: index as u32 + 1,
            reservation_id: request.id,
            id_tag: request.id_token.id_token.as_str().to_owned(),
            expires,
        };
        self.reservations
            .retain(|reservation| reservation.id != request.id);
        self.reservations.push(Reservation {
            id: request.id,
            index,
            expires,
            id_token: request.id_token,
            group: request.group_id_token.map(|group| group.id_token),
        });
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

    pub(super) fn reserved(&self, index: usize) -> Option<i32> {
        let reservation = self
            .reservations
            .iter()
            .find(|reservation| reservation.index == index);
        reservation.map(|reservation| reservation.id)
    }

    pub(super) fn reservation_allows(
        &self,
        index: usize,
        id_token: &IdToken,
        group: Option<&CiString<36>>,
    ) -> bool {
        let reservations = self.reservations.iter();
        reservations
            .filter(|reservation| reservation.index == index)
            .all(|reservation| reservation.admits(id_token, group))
    }

    pub(super) fn claim_reservation(
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
        let position = self
            .reservations
            .iter()
            .position(|reservation| reservation.index == index);
        if let Some(position) = position {
            self.sessions[index].reservation_id = Some(self.reservations[position].id);
            self.end_reservation(out, now, position, ReservationEnd::Used);
        }
        true
    }

    pub(super) fn reservation_deadline(&self) -> Option<u64> {
        let expiries = self
            .reservations
            .iter()
            .map(|reservation| reservation.expires);
        expiries.min()
    }

    pub(super) fn reservations_tick(&mut self, out: &mut Vec<Output>, now: u64) {
        while let Some(position) = self.reservations.iter().position(|reservation| {
            now >= reservation.expires || !self.operative(reservation.index)
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
            evse: reservation.index as u32 + 1,
            reservation_id: reservation.id,
            reason,
        };
        self.emit(out, now, kind);
    }
}

impl Reservation {
    fn admits(&self, id_token: &IdToken, group: Option<&CiString<36>>) -> bool {
        let same_group = match (&self.group, group) {
            (Some(reserved), Some(group)) => reserved.matches(group),
            _ => false,
        };
        self.id_token.id_token.matches(&id_token.id_token) || same_group
    }
}

fn response(status: ReserveNowStatus) -> ReserveNowResponse {
    ReserveNowResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "reservation.test.rs"]
mod tests;
