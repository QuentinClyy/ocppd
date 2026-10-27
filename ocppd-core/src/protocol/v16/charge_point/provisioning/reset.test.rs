use crate::api::intent::EvSpec;
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::fixtures::v16::plugged::plugged;
use crate::fixtures::v16::replies::status;
use crate::protocol::v16::event::EventKind;
use serde_json::json;

fn start_in_flight() -> Csms {
    let mut csms = plugged();
    csms.intent(0, authorize("TAG1"));
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));
    csms
}

fn hard_reset_while_charging() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.call(MINUTE, "Reset", json!({"type": "Hard"}));
    csms
}

#[test]
fn given_charging_ev_when_soft_reset_then_accepted_stopped_as_soft_reset_and_closed_once_delivered()
{
    let mut csms = Csms::charging(0, EvSpec::default());

    let reply = csms.call(MINUTE, "Reset", json!({"type": "Soft"}));

    assert_eq!(status(reply), "Accepted");
    assert_eq!(csms.sent("StopTransaction")[0]["reason"], "SoftReset");
    assert!(
        csms.closed,
        "connection closed once StopTransaction was delivered"
    );
}

#[test]
fn given_soft_reset_when_reconnected_then_boots_again() {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.call(MINUTE, "Reset", json!({"type": "Soft"}));
    csms.input(MINUTE, Input::Disconnected);

    csms.input(MINUTE + 5_000, Input::Connected);

    csms.expect("BootNotification");
}

#[test]
fn given_no_transaction_when_hard_reset_then_accepted_and_closed_at_once() {
    let mut csms = Csms::booted();

    let reply = csms.call(0, "Reset", json!({"type": "Hard"}));

    assert_eq!(status(reply), "Accepted");
    assert!(csms.closed);
    assert!(csms.sent("StopTransaction").is_empty());
}

#[test]
fn given_start_transaction_in_flight_when_soft_reset_then_accepted_and_waits_for_the_reply() {
    let mut csms = start_in_flight();
    csms.expect("StartTransaction");

    let reply = csms.call(1_000, "Reset", json!({"type": "Soft"}));

    assert_eq!(status(reply), "Accepted");
    assert!(!csms.closed, "waits for the StartTransaction reply");
}

#[test]
fn given_reset_during_start_transaction_when_it_is_answered_then_late_transaction_stopped_and_closed()
 {
    let mut csms = start_in_flight();
    csms.call(1_000, "Reset", json!({"type": "Soft"}));

    csms.answer(
        2_000,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 8}),
    );

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["transactionId"], 8);
    assert_eq!(stop["reason"], "Other");
    assert!(
        !csms
            .events
            .iter()
            .any(|kind| matches!(kind, EventKind::TransactionStarted { .. }))
    );
    assert!(csms.closed);
}

#[test]
fn given_charging_ev_when_hard_reset_then_closed_at_once_without_stop_transaction() {
    let csms = hard_reset_while_charging();

    let stops = csms.sent("StopTransaction");

    assert!(csms.closed);
    assert!(stops.is_empty());
}

#[test]
fn given_hard_reset_while_charging_when_booted_again_then_transaction_stopped_as_hard_reset_at_the_reset_time()
 {
    let mut csms = hard_reset_while_charging();
    csms.disconnect(MINUTE);
    csms.input(MINUTE + 5_000, Input::Connected);

    csms.answer(
        MINUTE + 5_000,
        json!({"status": "Accepted", "currentTime": "2026-10-20T00:00:00Z", "interval": 86400}),
    );

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["reason"], "HardReset");
    assert_eq!(stop["timestamp"], "1970-01-01T00:01:00Z");
}

#[test]
fn given_soft_reset_waiting_for_stop_transaction_when_remote_start_then_rejected() {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.answered_by_test.push("StopTransaction");
    csms.call(MINUTE, "Reset", json!({"type": "Soft"}));
    let remote = json!({"connectorId": 1, "idTag": "R1"});

    let reply = csms.call(MINUTE, "RemoteStartTransaction", remote);

    assert!(!csms.closed);
    assert_eq!(status(reply), "Rejected");
}
