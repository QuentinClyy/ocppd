use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::fixtures::v201::transaction::plug_in;
use serde_json::{Value, json};

fn broken_lock(run: &mut Run) {
    run.input(0, Input::Intent(Intent::LockFailure { connector: 1 }));
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

fn unlock(run: &mut Run) -> Value {
    let unlock = json!({"evseId": 1, "connectorId": 1});
    run.result(0, "UnlockConnector", unlock)["status"].clone()
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

#[test]
fn given_broken_lock_when_ev_plugged_then_faulted_without_charging_and_lock_problem_notified_per_g05_fr_01_fr_02_and_tc_g_20()
 {
    let mut run = Run::accepted(86_400);
    broken_lock(&mut run);

    plug_in(&mut run, 0);

    assert_eq!(last_status(&run), "Faulted");
    assert!(sent(&run, "TransactionEvent").is_empty(), "G05.FR.01");
    let event = &sent(&run, "NotifyEvent")[0]["eventData"][0];
    assert_eq!(event["trigger"], "Delta", "G05.FR.02");
    assert_eq!(event["component"]["name"], "ConnectorPlugRetentionLock");
    assert_eq!(
        event["component"]["evse"],
        json!({"id": 1, "connectorId": 1})
    );
    assert_eq!(event["variable"]["name"], "Problem");
    assert_eq!(event["actualValue"], "true");
    assert_eq!(event["eventNotificationType"], "HardWiredNotification");
    assert!(event.get("variableMonitoringId").is_none());
}

#[test]
fn given_lock_failure_reported_when_fault_cleared_then_lock_problem_false() {
    let mut run = Run::accepted(86_400);
    broken_lock(&mut run);
    plug_in(&mut run, 0);
    let get = json!({"getVariableData": [{"component": {"name": "ConnectorPlugRetentionLock", "evse": {"id": 1, "connectorId": 1}}, "variable": {"name": "Problem"}}]});

    clear_fault(&mut run, 0);

    let read = run.result(0, "GetVariables", get)["getVariableResult"][0].clone();
    assert_eq!(read["attributeValue"], "false");
}

#[test]
fn given_broken_lock_when_connector_unlocked_then_unlock_failed_per_f05() {
    let mut run = Run::accepted(86_400);
    broken_lock(&mut run);

    let status = unlock(&mut run);

    assert_eq!(status, "UnlockFailed");
}

#[test]
fn given_repaired_lock_when_connector_unlocked_then_unlocked_per_f05() {
    let mut run = Run::accepted(86_400);
    broken_lock(&mut run);
    clear_fault(&mut run, 0);

    let status = unlock(&mut run);

    assert_eq!(status, "Unlocked");
}
