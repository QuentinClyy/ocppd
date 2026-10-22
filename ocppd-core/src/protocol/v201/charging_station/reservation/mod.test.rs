use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::reservation::{
    EXPIRY, EXPIRY_MS, accepted, authorize, reserve, sent, statuses,
};
use crate::fixtures::v201::station::Run;
use crate::protocol::v201::event::{EventKind, ReservationEnd};
use serde_json::{Value, json};

fn plugged_with_authorized_start() -> Run {
    let mut run = Run::accepted_with(2, 86_400);
    assert!(run.station.configure("TxCtrlr.TxStartPoint", "Authorized"));
    let ev = EvSpec::default();
    run.input(0, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(0);
    run
}

#[test]
fn given_available_evse_when_reserved_then_accepted_and_reserved_per_h01_fr_03_and_fr_23_and_tc_h_01()
 {
    let mut run = Run::accepted(86_400);

    let status = reserve(&mut run, 1, Some(1), "04A2B3C4");

    assert_eq!(status, "Accepted");
    assert_eq!(statuses(&run), vec![(1, "Reserved".to_owned())]);
    assert!(run.events.contains(&EventKind::Reserved {
        evse: Some(1),
        reservation_id: 1,
        id_tag: "04A2B3C4".into(),
        expires: EXPIRY_MS,
    }));
}

#[test]
fn given_evse_in_a_transaction_when_reserved_then_occupied_per_h01_fr_11_and_tc_h_04() {
    let mut run = Run::accepted_with(2, 86_400);
    authorize(&mut run, 1, "0A0B0C0D", accepted());

    let status = reserve(&mut run, 1, Some(1), "04A2B3C4");

    assert_eq!(status, "Occupied");
}

#[test]
fn given_plugged_evse_when_reserved_then_occupied_per_h01_fr_11() {
    let mut run = plugged_with_authorized_start();

    let status = reserve(&mut run, 1, Some(1), "04A2B3C4");

    assert_eq!(status, "Occupied");
}

#[test]
fn given_reserved_evse_when_reserved_for_another_id_token_then_occupied_per_h01_fr_11_and_tc_h_03()
{
    let mut run = plugged_with_authorized_start();
    reserve(&mut run, 2, Some(2), "04A2B3C4");

    let status = reserve(&mut run, 3, Some(2), "0A0B0C0D");

    assert_eq!(status, "Occupied");
}

#[test]
fn given_inoperative_evse_when_reserved_then_unavailable_per_h01_fr_14_and_tc_h_06() {
    let mut run = Run::accepted_with(2, 86_400);
    let inoperative = json!({"operationalStatus": "Inoperative", "evse": {"id": 2}});
    run.result(0, "ChangeAvailability", inoperative);

    let status = reserve(&mut run, 3, Some(2), "0A0B0C0D");

    assert_eq!(status, "Unavailable");
}

#[test]
fn given_one_evse_when_evse_2_reserved_then_rejected() {
    let mut run = Run::accepted(86_400);

    let status = reserve(&mut run, 1, Some(2), "04A2B3C4");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_no_unspecified_reservations_when_reserved_without_evse_then_rejected_per_h01_fr_19() {
    let mut run = Run::accepted(86_400);
    assert!(
        run.station
            .configure("ReservationCtrlr.NonEvseSpecific", "false")
    );

    let status = reserve(&mut run, 1, None, "04A2B3C4");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_reservations_disabled_when_evse_reserved_then_rejected_per_h01_fr_01_and_tc_h_22() {
    let mut run = Run::accepted(86_400);
    assert!(run.station.configure("ReservationCtrlr.Enabled", "false"));

    let status = reserve(&mut run, 1, Some(1), "04A2B3C4");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_reservation_when_its_id_reserves_its_evse_for_another_id_token_then_replaced_per_h01_fr_02_and_tc_h_23()
 {
    let mut run = Run::accepted_with(2, 86_400);
    reserve(&mut run, 1, Some(1), "04A2B3C4");

    let status = reserve(&mut run, 1, Some(1), "0A0B0C0D");

    assert_eq!(status, "Accepted");
}

#[test]
fn given_reservation_when_its_id_reserves_another_evse_then_moved_per_h01_fr_02() {
    let mut run = Run::accepted_with(2, 86_400);
    reserve(&mut run, 1, Some(1), "04A2B3C4");

    let status = reserve(&mut run, 1, Some(2), "04A2B3C4");

    assert_eq!(status, "Accepted");
    assert_eq!(
        statuses(&run),
        vec![
            (1, "Reserved".to_owned()),
            (1, "Available".to_owned()),
            (2, "Reserved".to_owned())
        ]
    );
}

fn reserved() -> Run {
    let mut run = Run::accepted(86_400);
    reserve(&mut run, 1, Some(1), "04A2B3C4");
    run
}

fn cancel(run: &mut Run, id: i32) -> Value {
    let status = run.result(0, "CancelReservation", json!({"reservationId": id}))["status"].clone();
    run.answer_all(0);
    status
}

fn remote_start(run: &mut Run, id_token: Value) -> Value {
    let request = json!({"idToken": id_token, "remoteStartId": 7, "evseId": 1});
    let status = run.result(0, "RequestStartTransaction", request)["status"].clone();
    run.answer_all(0);
    status
}

#[test]
fn given_reservation_when_unknown_reservation_cancelled_then_rejected_per_h02_fr_01_and_tc_h_18() {
    let mut run = reserved();

    let status = cancel(&mut run, 9);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_reservation_when_cancelled_then_accepted_and_evse_available_per_h02_fr_02_to_fr_04_and_tc_h_17()
 {
    let mut run = reserved();

    let status = cancel(&mut run, 1);

    assert_eq!(status, "Accepted");
    assert_eq!(statuses(&run).last().unwrap().1, "Available");
    assert!(run.events.contains(&EventKind::ReservationEnded {
        evse: Some(1),
        reservation_id: 1,
        reason: ReservationEnd::Cancelled
    }));
}

#[test]
fn given_reservation_when_scheduled_then_due_at_its_expiry_per_h04_fr_01() {
    let run = reserved();

    let deadline = run.station.next_deadline();

    assert_eq!(deadline, Some(EXPIRY_MS));
}

#[test]
fn given_reservation_when_it_expires_then_reported_expired_and_evse_available_per_h04_fr_01_to_fr_03_and_tc_h_07()
 {
    let mut run = reserved();

    run.input(EXPIRY_MS, Input::Tick);
    run.answer_all(EXPIRY_MS);

    assert_eq!(
        sent(&run, "ReservationStatusUpdate"),
        vec![json!({"reservationId": 1, "reservationUpdateStatus": "Expired"})]
    );
    assert_eq!(statuses(&run).last().unwrap().1, "Available");
}

#[test]
fn given_reserved_evse_when_made_inoperative_then_reservation_removed_per_h01_fr_17_and_tc_h_21() {
    let mut run = reserved();
    let inoperative = json!({"operationalStatus": "Inoperative", "evse": {"id": 1}});

    run.result(0, "ChangeAvailability", inoperative);
    run.answer_all(0);

    assert_eq!(
        sent(&run, "ReservationStatusUpdate"),
        vec![json!({"reservationId": 1, "reservationUpdateStatus": "Removed"})]
    );
}

#[test]
fn given_reserved_evse_when_another_id_token_accepted_then_no_transaction_per_h03_fr_01_and_tc_h_02()
 {
    let mut run = reserved();

    authorize(&mut run, 1, "0A0B0C0D", accepted());

    assert!(sent(&run, "TransactionEvent").is_empty());
}

#[test]
fn given_reserved_evse_and_cached_other_id_token_when_presented_again_then_refused_from_the_cache_per_h03_fr_01()
 {
    let mut run = reserved();
    authorize(&mut run, 1, "0A0B0C0D", accepted());
    let cached = Intent::Authorize {
        connector: Some(1),
        id_tag: "0A0B0C0D".into(),
    };

    run.input(0, Input::Intent(cached));

    assert_eq!(sent(&run, "Authorize").len(), 1, "decided from the cache");
    assert!(sent(&run, "TransactionEvent").is_empty());
}

#[test]
fn given_reserved_evse_when_reserved_id_token_accepted_then_transaction_uses_the_reservation_per_h01_fr_15_and_tc_h_01()
 {
    let mut run = reserved();

    authorize(&mut run, 1, "04A2B3C4", accepted());

    let started = &sent(&run, "TransactionEvent")[0];
    assert_eq!(started["reservationId"], 1);
    assert!(run.events.contains(&EventKind::ReservationEnded {
        evse: Some(1),
        reservation_id: 1,
        reason: ReservationEnd::Used
    }));
}

#[test]
fn given_group_reservation_when_member_of_the_group_accepted_then_transaction_uses_the_reservation_per_h03_fr_04_and_tc_h_19()
 {
    let mut run = Run::accepted(86_400);
    let request = json!({
        "id": 1,
        "expiryDateTime": EXPIRY,
        "idToken": {"idToken": "04A2B3C4", "type": "ISO14443"},
        "groupIdToken": {"idToken": "FLEET", "type": "Central"},
        "evseId": 1,
    });
    run.result(0, "ReserveNow", request);
    run.answer_all(0);
    let member = json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "FLEET", "type": "Central"}}});

    authorize(&mut run, 1, "0A0B0C0D", member);

    assert_eq!(sent(&run, "TransactionEvent")[0]["reservationId"], 1);
}

#[test]
fn given_reserved_evse_when_remote_start_for_another_id_token_then_rejected_per_f01_fr_21_and_tc_h_02()
 {
    let mut run = reserved();

    let status = remote_start(
        &mut run,
        json!({"idToken": "APP-USER-1", "type": "Central"}),
    );

    assert_eq!(status, "Rejected");
}

#[test]
fn given_reserved_evse_when_remote_start_for_the_reserved_id_token_then_transaction_uses_the_reservation_per_f01_fr_22()
 {
    let mut run = reserved();

    let status = remote_start(&mut run, json!({"idToken": "04A2B3C4", "type": "ISO14443"}));

    assert_eq!(status, "Accepted");
    assert_eq!(sent(&run, "TransactionEvent")[0]["reservationId"], 1);
}
