use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::availability::ConnectorStatus;
use serde_json::{Value, json};

fn change(run: &mut Run, request: Value) -> Value {
    let status = run.result(0, "ChangeAvailability", request)["status"].clone();
    run.answer_all(0);
    status
}

fn statuses(run: &Run) -> Vec<ConnectorStatus> {
    run.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::StatusChanged { status, .. } => Some(*status),
            _ => None,
        })
        .collect()
}

fn plug_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
}

fn inoperative_evse() -> Value {
    json!({"operationalStatus": "Inoperative", "evse": {"id": 1}})
}

fn status_notifications(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "StatusNotification")
        .map(|(_, _, payload)| payload["connectorStatus"].clone())
        .collect()
}

#[test]
fn given_booted_station_when_evse_made_inoperative_then_accepted_and_unavailable_per_g03_fr_04_and_fr_06_and_tc_g_03()
 {
    let mut run = Run::accepted(86_400);

    let status = change(&mut run, inoperative_evse());

    assert_eq!(status, "Accepted");
    assert_eq!(statuses(&run), vec![ConnectorStatus::Unavailable]);
}

#[test]
fn given_inoperative_evse_when_made_inoperative_again_then_accepted_per_g03_fr_03_and_tc_g_10() {
    let mut run = Run::accepted(86_400);
    change(&mut run, inoperative_evse());

    let status = change(&mut run, inoperative_evse());

    assert_eq!(status, "Accepted");
}

#[test]
fn given_inoperative_evse_when_its_connector_made_operative_then_available_per_g03_fr_07_and_tc_g_08()
 {
    let mut run = Run::accepted(86_400);
    change(&mut run, inoperative_evse());
    let operative = json!({"operationalStatus": "Operative", "evse": {"id": 1, "connectorId": 1}});

    let status = change(&mut run, operative);

    assert_eq!(status, "Accepted");
    assert_eq!(statuses(&run).last(), Some(&ConnectorStatus::Available));
}

#[test]
fn given_one_evse_with_one_connector_when_unknown_evse_or_connector_changed_then_rejected() {
    let mut run = Run::accepted(86_400);
    let unknown = [json!({"id": 2}), json!({"id": 1, "connectorId": 2})];

    let statuses: Vec<_> = unknown
        .iter()
        .map(|evse| {
            change(
                &mut run,
                json!({"operationalStatus": "Inoperative", "evse": evse}),
            )
        })
        .collect();

    assert_eq!(statuses, vec![json!("Rejected"), json!("Rejected")]);
}

#[test]
fn given_transaction_when_evse_made_inoperative_then_scheduled_and_still_occupied_per_g03_fr_05_and_tc_g_11()
 {
    let mut run = Run::accepted(86_400);
    plug_in(&mut run, 0);

    let status = change(&mut run, inoperative_evse());

    assert_eq!(status, "Scheduled");
    assert_eq!(statuses(&run), vec![ConnectorStatus::Occupied]);
}

#[test]
fn given_scheduled_inoperative_evse_when_transaction_ends_then_unavailable_per_g03_fr_05_and_tc_g_11()
 {
    let mut run = Run::accepted(86_400);
    plug_in(&mut run, 0);
    change(&mut run, inoperative_evse());

    run.input(1_000, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(1_000);

    assert_eq!(statuses(&run).last(), Some(&ConnectorStatus::Unavailable));
}

#[test]
fn given_two_evses_when_station_made_inoperative_then_both_unavailable_per_g04_fr_01_and_fr_07_and_tc_g_05()
 {
    let mut run = Run::accepted_with(2, 86_400);

    let status = change(&mut run, json!({"operationalStatus": "Inoperative"}));

    assert_eq!(status, "Accepted");
    assert_eq!(
        statuses(&run),
        vec![ConnectorStatus::Unavailable, ConnectorStatus::Unavailable]
    );
}

#[test]
fn given_inoperative_station_when_ev_plugged_then_no_transaction_per_g04_fr_01() {
    let mut run = Run::accepted_with(2, 86_400);
    change(&mut run, json!({"operationalStatus": "Inoperative"}));

    plug_in(&mut run, 0);

    assert!(!run.actions().contains(&"TransactionEvent".to_owned()));
}

#[test]
fn given_inoperative_station_when_made_inoperative_again_then_every_connector_reported_again_per_g04_fr_04_and_tc_g_13()
 {
    let mut run = Run::accepted_with(2, 86_400);
    let inoperative = json!({"operationalStatus": "Inoperative"});
    change(&mut run, inoperative.clone());

    let status = change(&mut run, inoperative);

    assert_eq!(status, "Accepted");
    assert_eq!(status_notifications(&run), vec![json!("Unavailable"); 4]);
}

fn fault(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::Fault { connector: 1 }));
    run.answer_all(now);
}

fn sent(run: &Run, action: &str) -> Vec<Value> {
    let calls = run.calls.iter().filter(|(_, sent, _)| sent == action);
    calls.map(|(_, _, payload)| payload.clone()).collect()
}

fn last_status(run: &Run) -> Value {
    sent(run, "StatusNotification").pop().unwrap()["connectorStatus"].clone()
}

fn reserve(run: &mut Run, id: i32) -> Value {
    let request = json!({"id": id, "expiryDateTime": "1970-01-01T01:00:00Z", "idToken": {"idToken": "04A2B3C4", "type": "ISO14443"}, "evseId": 1});
    let status = run.result(0, "ReserveNow", request)["status"].clone();
    run.answer_all(0);
    status
}

fn clear_fault(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::ClearFault { connector: 1 }));
    run.answer_all(now);
}

#[test]
fn given_available_evse_when_faulted_then_faulted_reported() {
    let mut run = Run::accepted(86_400);

    fault(&mut run, 0);

    assert_eq!(last_status(&run), "Faulted");
}

#[test]
fn given_faulted_evse_when_ev_plugged_then_no_transaction() {
    let mut run = Run::accepted(86_400);
    fault(&mut run, 0);

    plug_in(&mut run, MINUTE);

    assert!(sent(&run, "TransactionEvent").is_empty());
}

#[test]
fn given_faulted_evse_with_plugged_ev_when_fault_cleared_then_occupied() {
    let mut run = Run::accepted(86_400);
    fault(&mut run, 0);
    plug_in(&mut run, MINUTE);

    clear_fault(&mut run, 2 * MINUTE);

    assert_eq!(last_status(&run), "Occupied");
}

#[test]
fn given_transaction_when_evse_faulted_then_ended_as_abnormal_condition() {
    let mut run = Run::accepted(86_400);
    plug_in(&mut run, 0);

    fault(&mut run, MINUTE);

    let ended = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(ended["eventType"], "Ended");
    assert_eq!(ended["triggerReason"], "AbnormalCondition");
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "Other");
}

#[test]
fn given_reserved_evse_when_faulted_then_reservation_removed_per_h01_fr_16() {
    let mut run = Run::accepted(86_400);
    reserve(&mut run, 1);

    fault(&mut run, 0);

    assert_eq!(
        sent(&run, "ReservationStatusUpdate"),
        vec![json!({"reservationId": 1, "reservationUpdateStatus": "Removed"})]
    );
}

#[test]
fn given_faulted_evse_when_reserved_then_faulted_per_h01_fr_12() {
    let mut run = Run::accepted(86_400);
    fault(&mut run, 0);

    let status = reserve(&mut run, 2);

    assert_eq!(status, "Faulted");
}

#[test]
fn given_faulted_evse_when_remote_start_then_rejected_per_f01_fr_23() {
    let mut run = Run::accepted(86_400);
    fault(&mut run, 0);
    let request = json!({"idToken": {"idToken": "APP-USER-1", "type": "Central"}, "remoteStartId": 7, "evseId": 1});

    let status = run.result(0, "RequestStartTransaction", request)["status"].clone();

    assert_eq!(status, "Rejected");
}
