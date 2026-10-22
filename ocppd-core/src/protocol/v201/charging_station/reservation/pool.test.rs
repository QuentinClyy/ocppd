use crate::fixtures::v201::reservation::{EXPIRY, accepted, authorize, reserve, sent, statuses};
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn reserve_type(run: &mut Run, id: i32, evse: Option<u32>, connector_type: &str) -> Value {
    let mut request = json!({
        "id": id,
        "expiryDateTime": EXPIRY,
        "idToken": {"idToken": "04A2B3C4", "type": "ISO14443"},
        "connectorType": connector_type,
    });
    if let Some(evse) = evse {
        request["evseId"] = json!(evse);
    }
    run.result(0, "ReserveNow", request)["status"].clone()
}

fn reserved(run: &Run) -> Vec<u64> {
    let statuses = statuses(run).into_iter();
    let reserved = statuses.filter(|(_, status)| status == "Reserved");
    reserved.map(|(evse, _)| evse).collect()
}

fn two_evses_one_floating() -> Run {
    let mut run = Run::accepted_with(2, 86_400);
    reserve(&mut run, 1, None, "04A2B3C4");
    run
}

fn two_evses_two_floating() -> Run {
    let mut run = two_evses_one_floating();
    reserve(&mut run, 2, None, "0A0B0C0D");
    run
}

fn transactions(run: &Run) -> usize {
    sent(run, "TransactionEvent").len()
}

#[test]
fn given_two_free_evses_when_one_unspecified_reservation_then_accepted_and_none_reserved_yet_per_h01_fr_04_fr_18_and_tc_h_08()
 {
    let mut run = Run::accepted_with(2, 86_400);

    let status = reserve(&mut run, 1, None, "04A2B3C4");

    assert_eq!(status, "Accepted");
    assert_eq!(reserved(&run), Vec::<u64>::new());
}

#[test]
fn given_one_floating_reservation_when_second_unspecified_reservation_then_every_evse_reserved_per_h01_fr_20_and_tc_h_14()
 {
    let mut run = two_evses_one_floating();

    let status = reserve(&mut run, 2, None, "0A0B0C0D");

    assert_eq!(status, "Accepted");
    assert_eq!(reserved(&run), vec![1, 2]);
}

#[test]
fn given_every_evse_reserved_when_unspecified_reservation_then_occupied_per_h01_fr_11_and_tc_h_09()
{
    let mut run = two_evses_two_floating();

    let status = reserve(&mut run, 3, None, "0E0E0E0E");

    assert_eq!(status, "Occupied");
}

#[test]
fn given_one_floating_reservation_when_evse_1_reserved_then_accepted_per_h01_fr_07() {
    let mut run = two_evses_one_floating();

    let status = reserve(&mut run, 2, Some(1), "0A0B0C0D");

    assert_eq!(status, "Accepted");
}

#[test]
fn given_floating_and_evse_1_reservations_when_last_free_evse_reserved_then_occupied_per_h01_fr_07()
{
    let mut run = two_evses_one_floating();
    reserve(&mut run, 2, Some(1), "0A0B0C0D");

    let status = reserve(&mut run, 3, Some(2), "0E0E0E0E");

    assert_eq!(status, "Occupied");
}

#[test]
fn given_one_floating_reservation_when_another_driver_authorized_then_charges_on_a_free_evse_per_h03_fr_03()
 {
    let mut run = two_evses_one_floating();

    authorize(&mut run, 1, "0A0B0C0D", accepted());

    assert_eq!(transactions(&run), 1);
}

#[test]
fn given_floating_reservation_and_one_other_driver_when_a_third_driver_authorized_then_last_free_evse_held_per_h01_fr_07()
 {
    let mut run = two_evses_one_floating();
    authorize(&mut run, 1, "0A0B0C0D", accepted());

    authorize(&mut run, 2, "0E0E0E0E", accepted());

    assert_eq!(transactions(&run), 1);
}

#[test]
fn given_held_last_evse_when_the_reserved_driver_authorized_then_charges_on_it_with_the_reservation_per_h03_fr_03()
 {
    let mut run = two_evses_one_floating();
    authorize(&mut run, 1, "0A0B0C0D", accepted());

    authorize(&mut run, 2, "04A2B3C4", accepted());

    let started = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(started["evse"]["id"], 2);
    assert_eq!(started["reservationId"], 1);
}

#[test]
fn given_only_type_2_connectors_when_ccs_reserved_without_evse_then_rejected_per_h01_fr_06() {
    let mut run = Run::accepted(86_400);

    let status = reserve_type(&mut run, 1, None, "cCCS2");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_type_2_evse_when_ccs_reserved_on_it_then_rejected_per_h01_fr_06() {
    let mut run = Run::accepted(86_400);

    let status = reserve_type(&mut run, 1, Some(1), "cCCS2");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_one_type_2_evse_when_type_2_reserved_then_accepted_and_its_evse_reserved_per_h01_fr_06_fr_24_and_tc_h_15()
 {
    let mut run = Run::accepted(86_400);

    let status = reserve_type(&mut run, 1, None, "cType2");
    run.answer_all(0);

    assert_eq!(status, "Accepted");
    assert_eq!(reserved(&run), vec![1]);
}

#[test]
fn given_type_2_reservation_when_its_driver_authorized_then_transaction_uses_it_per_h03_fr_02() {
    let mut run = Run::accepted(86_400);
    reserve_type(&mut run, 1, None, "cType2");
    run.answer_all(0);

    authorize(&mut run, 1, "04A2B3C4", accepted());

    assert_eq!(sent(&run, "TransactionEvent")[0]["reservationId"], 1);
}

#[test]
fn given_floating_reservation_when_station_made_inoperative_then_reservation_removed_per_h01_fr_17()
{
    let mut run = two_evses_one_floating();

    run.result(
        0,
        "ChangeAvailability",
        json!({"operationalStatus": "Inoperative"}),
    );
    run.answer_all(0);

    assert_eq!(
        sent(&run, "ReservationStatusUpdate"),
        vec![json!({"reservationId": 1, "reservationUpdateStatus": "Removed"})]
    );
}

#[test]
fn given_inoperative_station_when_unspecified_reservation_then_unavailable_per_h01_fr_14_and_tc_h_12()
 {
    let mut run = Run::accepted_with(2, 86_400);
    run.result(
        0,
        "ChangeAvailability",
        json!({"operationalStatus": "Inoperative"}),
    );
    run.answer_all(0);

    let status = reserve(&mut run, 2, None, "04A2B3C4");

    assert_eq!(status, "Unavailable");
}

#[test]
fn given_every_evse_held_by_floating_reservations_when_one_cancelled_then_both_evses_available_again_per_h02_fr_05_and_fr_06()
 {
    let mut run = two_evses_two_floating();

    run.result(0, "CancelReservation", json!({"reservationId": 2}));
    run.answer_all(0);

    let statuses = statuses(&run);
    assert_eq!(
        statuses[statuses.len() - 2..],
        [(1, "Available".to_owned()), (2, "Available".to_owned())]
    );
}
