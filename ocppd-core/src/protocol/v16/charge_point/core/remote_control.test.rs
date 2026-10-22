use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::fixtures::v16::plugged::plugged;
use crate::fixtures::v16::replies::status;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::core::status::ChargePointStatus;
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

fn inoperative_connector() -> Csms {
    let mut csms = Csms::booted();
    let inoperative = json!({"connectorId": 1, "type": "Inoperative"});
    csms.call(0, "ChangeAvailability", inoperative);
    csms
}

fn plug_in(csms: &mut Csms) {
    csms.intent(
        0,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
}

fn connector_1_statuses(csms: &Csms) -> Vec<ChargePointStatus> {
    csms.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::StatusChanged {
                connector: 1,
                status,
            } => Some(*status),
            _ => None,
        })
        .collect()
}

fn scheduled_inoperative_station() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    let whole_station = json!({"connectorId": 0, "type": "Inoperative"});
    csms.call(MINUTE, "ChangeAvailability", whole_station);
    csms
}

#[test]
fn given_booted_charge_point_when_connector_made_inoperative_then_accepted_and_unavailable() {
    let mut csms = Csms::booted();
    let inoperative = json!({"connectorId": 1, "type": "Inoperative"});

    let reply = csms.call(0, "ChangeAvailability", inoperative);

    assert_eq!(status(reply), "Accepted");
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Unavailable]);
}

#[test]
fn given_inoperative_connector_with_plugged_ev_when_id_tag_presented_then_refused_as_unavailable() {
    let mut csms = inoperative_connector();
    plug_in(&mut csms);
    csms.events.clear();

    csms.intent(0, authorize("TAG1"));

    let rejected = EventKind::Link(LinkEvent::IntentRejected {
        connector: Some(1),
        reason: "unavailable".into(),
    });
    assert_eq!(csms.events, vec![rejected]);
}

#[test]
fn given_inoperative_connector_when_remote_start_then_rejected() {
    let mut csms = inoperative_connector();
    let remote = json!({"connectorId": 1, "idTag": "R1"});

    let reply = csms.call(0, "RemoteStartTransaction", remote);

    assert_eq!(status(reply), "Rejected");
}

#[test]
fn given_inoperative_connector_with_plugged_ev_when_made_operative_then_accepted_and_preparing() {
    let mut csms = inoperative_connector();
    plug_in(&mut csms);
    let operative = json!({"connectorId": 1, "type": "Operative"});

    let reply = csms.call(0, "ChangeAvailability", operative);

    assert_eq!(status(reply), "Accepted");
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Preparing));
}

#[test]
fn given_charging_ev_when_station_made_inoperative_then_scheduled_station_unavailable_and_connector_still_charging()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    let whole_station = json!({"connectorId": 0, "type": "Inoperative"});

    let reply = csms.call(MINUTE, "ChangeAvailability", whole_station);

    assert_eq!(status(reply), "Scheduled");
    assert_eq!(
        csms.statuses().last(),
        Some(&ChargePointStatus::Unavailable)
    );
    assert_eq!(
        connector_1_statuses(&csms).last(),
        Some(&ChargePointStatus::Charging)
    );
}

#[test]
fn given_scheduled_inoperative_station_when_transaction_stopped_then_unavailable() {
    let mut csms = scheduled_inoperative_station();

    csms.intent(2 * MINUTE, Intent::Stop { connector: 1 });

    assert_eq!(
        csms.statuses().last(),
        Some(&ChargePointStatus::Unavailable)
    );
}

#[test]
fn given_plugged_ev_when_remote_start_on_its_connector_then_accepted_and_started_without_authorize()
{
    let mut csms = plugged();

    let reply = csms.call(
        0,
        "RemoteStartTransaction",
        json!({"connectorId": 1, "idTag": "R1"}),
    );

    assert_eq!(status(reply), "Accepted");
    let start = csms.expect("StartTransaction");
    assert_eq!(start["idTag"], "R1");
    assert!(csms.sent("Authorize").is_empty());
    assert!(csms.events.contains(&EventKind::RemoteStart {
        connector: Some(1),
        id_tag: "R1".into(),
        expires: None
    }));
}

#[test]
fn given_remote_started_transaction_when_start_transaction_answered_then_transaction_started() {
    let mut csms = plugged();
    csms.call(
        0,
        "RemoteStartTransaction",
        json!({"connectorId": 1, "idTag": "R1"}),
    );

    csms.answer(
        0,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 5}),
    );

    assert!(csms.events.contains(&EventKind::TransactionStarted {
        connector: 1,
        transaction_id: 5
    }));
}

#[test]
fn given_plugged_ev_when_remote_start_without_connector_then_accepted_on_the_plugged_one() {
    let mut csms = plugged();

    let reply = csms.call(0, "RemoteStartTransaction", json!({"idTag": "R1"}));

    assert_eq!(status(reply), "Accepted");
}

#[test]
fn given_one_connector_when_remote_start_on_connector_9_then_rejected() {
    let mut csms = Csms::booted();
    let unknown = json!({"connectorId": 9, "idTag": "R1"});

    let reply = csms.call(0, "RemoteStartTransaction", unknown);

    assert_eq!(status(reply), "Rejected");
}

#[test]
fn given_transaction_42_when_remote_stop_of_99_then_rejected() {
    let mut csms = Csms::charging(0, EvSpec::default());

    let reply = csms.call(
        MINUTE,
        "RemoteStopTransaction",
        json!({"transactionId": 99}),
    );

    assert_eq!(status(reply), "Rejected");
}

#[test]
fn given_transaction_42_when_remote_stop_of_42_then_accepted_and_stopped_as_remote() {
    let mut csms = Csms::charging(0, EvSpec::default());

    let reply = csms.call(
        MINUTE,
        "RemoteStopTransaction",
        json!({"transactionId": 42}),
    );

    assert_eq!(status(reply), "Accepted");
    assert_eq!(csms.sent("StopTransaction")[0]["reason"], "Remote");
}

#[test]
fn given_charging_ev_when_connector_unlocked_then_unlocked_and_stopped_as_unlock_command() {
    let mut csms = Csms::charging(0, EvSpec::default());

    let reply = csms.call(MINUTE, "UnlockConnector", json!({"connectorId": 1}));

    assert_eq!(status(reply), "Unlocked");
    assert_eq!(csms.sent("StopTransaction")[0]["reason"], "UnlockCommand");
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Finishing));
}

#[test]
fn given_one_connector_when_connector_7_unlocked_then_not_supported_per_tc_031() {
    let mut csms = Csms::booted();

    let reply = csms.call(MINUTE, "UnlockConnector", json!({"connectorId": 7}));

    assert_eq!(status(reply), "NotSupported");
}

#[test]
fn given_booted_charge_point_when_cache_cleared_then_accepted() {
    let mut csms = Csms::booted();

    let reply = csms.call(0, "ClearCache", json!({}));

    assert_eq!(status(reply), "Accepted");
}

#[test]
fn given_no_vendor_extensions_when_data_transfer_received_then_unknown_vendor_id() {
    let mut csms = Csms::booted();
    let transfer = json!({"vendorId": "com.example", "messageId": "x"});

    let reply = csms.call(0, "DataTransfer", transfer);

    assert_eq!(status(reply), "UnknownVendorId");
}
