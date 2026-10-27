use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::protocol::v16::event::{EventKind, ReservationEnd};
use crate::protocol::v16::messages::availability::ChargePointStatus;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn status(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { payload, .. } => payload["status"].clone(),
        other => panic!("expected a result, got {other:?}"),
    }
}

fn reserve(csms: &mut Csms, connector: u32, id: i32, extra: Value) -> Value {
    let mut request = json!({
        "connectorId": connector,
        "expiryDate": "1970-01-01T00:05:00Z",
        "idTag": "TAG1",
        "reservationId": id,
    });
    if let (Some(request), Some(extra)) = (request.as_object_mut(), extra.as_object()) {
        request.extend(extra.clone());
    }
    status(csms.call(0, "ReserveNow", request))
}

fn plug_in(csms: &mut Csms, connector: u32) {
    let ev = EvSpec::default();
    csms.intent(0, Intent::PlugIn { connector, ev });
}

fn ended(csms: &Csms) -> Vec<ReservationEnd> {
    csms.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::ReservationEnded { reason, .. } => Some(*reason),
            _ => None,
        })
        .collect()
}

fn reserved() -> Csms {
    let mut csms = Csms::booted();
    reserve(&mut csms, 1, 7, json!({}));
    csms
}

fn reserved_and_plugged() -> Csms {
    let mut csms = reserved();
    plug_in(&mut csms, 1);
    csms
}

fn cancel(csms: &mut Csms, id: i32) -> Value {
    status(csms.call(0, "CancelReservation", json!({"reservationId": id})))
}

fn remote_start(csms: &mut Csms, id_tag: &str) -> Value {
    let request = json!({"connectorId": 1, "idTag": id_tag});
    status(csms.call(0, "RemoteStartTransaction", request))
}

#[test]
fn given_available_connector_when_reserved_then_accepted_and_reserved() {
    let mut csms = Csms::booted();

    let status = reserve(&mut csms, 1, 7, json!({}));

    assert_eq!(status, "Accepted");
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Reserved]);
    assert!(csms.events.contains(&EventKind::Reserved {
        connector: 1,
        reservation_id: 7,
        id_tag: "TAG1".into(),
        expires: 5 * MINUTE,
    }));
}

#[test]
fn given_reserved_connector_with_plugged_ev_when_another_id_tag_accepted_then_refused_as_reserved()
{
    let mut csms = reserved_and_plugged();
    csms.intent(0, authorize("TAG2"));

    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    assert!(csms.sent("StartTransaction").is_empty());
    assert!(
        csms.events
            .contains(&EventKind::Link(LinkEvent::IntentRejected {
                connector: Some(1),
                reason: "reserved".into(),
            }))
    );
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Reserved]);
}

#[test]
fn given_reserved_connector_with_plugged_ev_when_the_reserved_id_tag_accepted_then_started_with_the_reservation()
 {
    let mut csms = reserved_and_plugged();
    csms.intent(0, authorize("tag1"));

    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    assert_eq!(csms.expect("StartTransaction")["reservationId"], 7);
    assert_eq!(ended(&csms), vec![ReservationEnd::Used]);
    assert_eq!(
        csms.statuses(),
        vec![ChargePointStatus::Reserved, ChargePointStatus::Preparing]
    );
}

#[test]
fn given_reservation_for_a_parent_when_an_id_tag_of_that_parent_accepted_then_started_with_the_reservation()
 {
    let mut csms = Csms::booted();
    reserve(&mut csms, 1, 7, json!({"parentIdTag": "FLEET"}));
    plug_in(&mut csms, 1);
    csms.intent(0, authorize("TAG2"));
    let info = json!({"idTagInfo": {"status": "Accepted", "parentIdTag": "FLEET"}});

    csms.answer(0, info);

    assert_eq!(csms.expect("StartTransaction")["reservationId"], 7);
}

#[test]
fn given_booted_charge_point_when_reserved_then_expiry_scheduled() {
    let csms = reserved();

    let deadline = csms.charge_point.next_deadline();

    assert_eq!(deadline, Some(5 * MINUTE));
}

#[test]
fn given_reservation_when_it_expires_then_ended_and_connector_available() {
    let mut csms = reserved();

    csms.input(5 * MINUTE, Input::Tick);

    assert_eq!(ended(&csms), vec![ReservationEnd::Expired]);
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Available));
}

#[test]
fn given_reservation_7_when_reservation_8_cancelled_then_rejected() {
    let mut csms = reserved();

    let status = cancel(&mut csms, 8);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_reservation_when_cancelled_then_accepted_ended_and_connector_available() {
    let mut csms = reserved();

    let status = cancel(&mut csms, 7);

    assert_eq!(status, "Accepted");
    assert_eq!(ended(&csms), vec![ReservationEnd::Cancelled]);
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Available));
}

#[test]
fn given_plugged_ev_when_its_connector_reserved_then_occupied() {
    let mut csms = Csms::booted();
    plug_in(&mut csms, 1);

    let status = reserve(&mut csms, 1, 7, json!({}));

    assert_eq!(status, "Occupied");
}

#[test]
fn given_reserved_connector_when_reserved_again_with_another_id_then_occupied() {
    let mut csms = reserved();

    let status = reserve(&mut csms, 1, 8, json!({}));

    assert_eq!(status, "Occupied");
}

#[test]
fn given_no_reservations_on_connector_0_when_connector_0_reserved_then_rejected() {
    let mut csms = Csms::booted();

    let status = reserve(&mut csms, 0, 7, json!({}));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_one_connector_when_connector_2_reserved_then_rejected() {
    let mut csms = Csms::booted();

    let status = reserve(&mut csms, 2, 7, json!({}));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_inoperative_connector_when_reserved_then_unavailable() {
    let mut csms = Csms::booted();
    let request = json!({"connectorId": 1, "type": "Inoperative"});
    status(csms.call(0, "ChangeAvailability", request));

    let status = reserve(&mut csms, 1, 7, json!({}));

    assert_eq!(status, "Unavailable");
}

#[test]
fn given_reservation_7_when_7_reserved_for_another_id_tag_then_replaced() {
    let mut csms = Csms::booted_with(2);
    reserve(&mut csms, 1, 7, json!({}));

    let status = reserve(&mut csms, 1, 7, json!({"idTag": "TAG2"}));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_reservation_7_on_connector_1_when_7_reserved_on_connector_2_then_moved() {
    let mut csms = Csms::booted_with(2);
    reserve(&mut csms, 1, 7, json!({}));
    reserve(&mut csms, 1, 7, json!({"idTag": "TAG2"}));

    let status = reserve(&mut csms, 2, 7, json!({}));

    assert_eq!(status, "Accepted");
    let statuses: Vec<_> = csms
        .sent("StatusNotification")
        .iter()
        .map(|sent| (sent["connectorId"].clone(), sent["status"].clone()))
        .collect();
    assert_eq!(
        statuses,
        vec![
            (json!(1), json!("Reserved")),
            (json!(1), json!("Available")),
            (json!(2), json!("Reserved")),
        ]
    );
}

#[test]
fn given_reserved_connector_with_plugged_ev_when_remote_start_for_another_id_tag_then_rejected() {
    let mut csms = reserved_and_plugged();

    let status = remote_start(&mut csms, "TAG2");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_reserved_connector_with_plugged_ev_when_remote_start_for_the_reserved_id_tag_then_started_with_the_reservation()
 {
    let mut csms = reserved_and_plugged();

    let status = remote_start(&mut csms, "TAG1");

    assert_eq!(status, "Accepted");
    assert_eq!(csms.expect("StartTransaction")["reservationId"], 7);
}
