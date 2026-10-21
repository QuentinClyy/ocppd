use super::ChargePoint;
use crate::api::io::Output;
use crate::v16::event::{EventKind, ReservationEnd};
use crate::v16::messages::core::status::ChargePointStatus;
use crate::v16::messages::datatypes::generic_status::GenericStatus;
use crate::v16::messages::datatypes::id_tag::IdToken;
use crate::v16::messages::reservation::{
    CancelReservationRequest, ReservationStatus, ReserveNowRequest,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Reservation {
    id: i32,
    id_tag: IdToken,
    parent_id_tag: Option<IdToken>,
    expires: u64,
}

impl ChargePoint {
    pub(super) fn reserve_now(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ReserveNowRequest,
    ) -> ReservationStatus {
        let Some(index) = self.index_of(request.connector_id) else {
            return ReservationStatus::Rejected;
        };
        if self.unavailable(index) {
            return ReservationStatus::Unavailable;
        }
        let renewed = matches!(&self.reservations[index], Some(existing) if existing.id == request.reservation_id);
        if !renewed && self.status(index, now) != ChargePointStatus::Available {
            return ReservationStatus::Occupied;
        }
        for slot in &mut self.reservations {
            if matches!(slot, Some(moved) if moved.id == request.reservation_id) {
                *slot = None;
            }
        }
        let kind = EventKind::Reserved {
            connector: request.connector_id,
            reservation_id: request.reservation_id,
            id_tag: request.id_tag.as_str().to_owned(),
            expires: request.expiry_date.unix_ms,
        };
        self.emit(out, now, kind);
        self.reservations[index] = Some(Reservation {
            id: request.reservation_id,
            id_tag: request.id_tag,
            parent_id_tag: request.parent_id_tag,
            expires: request.expiry_date.unix_ms,
        });
        ReservationStatus::Accepted
    }

    pub(super) fn cancel_reservation(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: CancelReservationRequest,
    ) -> GenericStatus {
        let found = self.reservations.iter().position(
            |slot| matches!(slot, Some(reservation) if reservation.id == request.reservation_id),
        );
        let Some(index) = found else {
            return GenericStatus::Rejected;
        };
        self.end_reservation(out, now, index, ReservationEnd::Cancelled);
        GenericStatus::Accepted
    }

    pub(super) fn expire_reservations(&mut self, out: &mut Vec<Output>, now: u64) {
        for index in 0..self.reservations.len() {
            if matches!(&self.reservations[index], Some(reservation) if now >= reservation.expires)
            {
                self.end_reservation(out, now, index, ReservationEnd::Expired);
            }
        }
    }

    pub(super) fn use_reservation(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
    ) -> Option<i32> {
        let id = self.reservations[index].as_ref()?.id;
        self.end_reservation(out, now, index, ReservationEnd::Used);
        Some(id)
    }

    pub(super) fn reservation_deadline(&self) -> Option<u64> {
        self.reservations
            .iter()
            .flatten()
            .map(|reservation| reservation.expires)
            .min()
    }

    pub(super) fn is_reserved(&self, index: usize) -> bool {
        self.reservations[index].is_some()
    }

    pub(super) fn reserved_for_other(
        &self,
        index: usize,
        id_tag: &IdToken,
        parent_id_tag: Option<&IdToken>,
    ) -> bool {
        let Some(reservation) = &self.reservations[index] else {
            return false;
        };
        let by_parent = match (&reservation.parent_id_tag, parent_id_tag) {
            (Some(reserved), Some(presented)) => reserved.matches(presented),
            _ => false,
        };
        !reservation.id_tag.matches(id_tag) && !by_parent
    }

    fn end_reservation(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        reason: ReservationEnd,
    ) {
        let Some(reservation) = self.reservations[index].take() else {
            return;
        };
        let kind = EventKind::ReservationEnded {
            connector: index as u32 + 1,
            reservation_id: reservation.id,
            reason,
        };
        self.emit(out, now, kind);
    }
}

#[cfg(test)]
#[path = "reservation.test.rs"]
mod tests;
