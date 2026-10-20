use crate::event::EventKind;
use crate::event::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::fixtures::v16::plugged::plugged;
use crate::fixtures::v16::replies::status;
use crate::intent::{EvSpec, Intent};
use crate::v16::messages::status::ChargePointStatus;
use serde_json::json;

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

fn inoperative_connector() -> Csms {
    let mut csms = Csms::booted();
    let inoperative = json!({"connectorId": 1, "type": "Inoperative"});
    csms.call(0, "ChangeAvailability", inoperative);
    csms
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
fn given_inoperative_connector_when_remote_start_then_rejected() {
    let mut csms = inoperative_connector();
    let remote = json!({"connectorId": 1, "idTag": "R1"});

    let reply = csms.call(0, "RemoteStartTransaction", remote);

    assert_eq!(status(reply), "Rejected");
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
