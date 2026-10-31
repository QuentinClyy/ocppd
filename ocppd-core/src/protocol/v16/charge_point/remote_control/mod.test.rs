use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::fixtures::v16::plugged::plugged;
use crate::fixtures::v16::replies::status;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use serde_json::{Value, json};

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

fn authorizing_remote_starts(mut csms: Csms) -> Csms {
    assert_eq!(
        csms.configure(0, "AuthorizeRemoteTxRequests", "true"),
        "Accepted"
    );
    csms
}

fn remote_start_r1(csms: &mut Csms) -> Value {
    status(csms.call(
        0,
        "RemoteStartTransaction",
        json!({"connectorId": 1, "idTag": "R1"}),
    ))
}

#[test]
fn given_remote_starts_authorized_first_when_remote_start_on_a_plugged_connector_then_authorize_sent_before_any_start_per_5_11()
 {
    let mut csms = authorizing_remote_starts(plugged());

    let status = remote_start_r1(&mut csms);

    assert_eq!(status, "Accepted");
    assert_eq!(csms.expect("Authorize")["idTag"], "R1");
    assert!(csms.sent("StartTransaction").is_empty());
}

#[test]
fn given_remote_start_waiting_for_its_authorize_when_accepted_then_start_transaction_for_its_id_tag_per_5_11()
 {
    let mut csms = authorizing_remote_starts(plugged());
    remote_start_r1(&mut csms);

    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    assert_eq!(csms.expect("StartTransaction")["idTag"], "R1");
}

#[test]
fn given_remote_start_waiting_for_its_authorize_when_blocked_then_no_transaction_per_5_11() {
    let mut csms = authorizing_remote_starts(plugged());
    remote_start_r1(&mut csms);

    csms.answer(0, json!({"idTagInfo": {"status": "Blocked"}}));

    assert!(csms.sent("StartTransaction").is_empty());
}

#[test]
fn given_remote_starts_authorized_first_and_a_listed_id_tag_when_remote_started_then_started_without_authorize_per_5_11()
 {
    let mut csms = authorizing_remote_starts(plugged());
    assert_eq!(csms.configure(0, "LocalPreAuthorize", "true"), "Accepted");
    let list = json!({
        "listVersion": 1,
        "updateType": "Full",
        "localAuthorizationList": [{"idTag": "R1", "idTagInfo": {"status": "Accepted"}}],
    });
    csms.call(0, "SendLocalList", list);

    remote_start_r1(&mut csms);

    assert_eq!(csms.expect("StartTransaction")["idTag"], "R1");
    assert!(csms.sent("Authorize").is_empty());
}

#[test]
fn given_remote_starts_authorized_first_and_no_ev_when_authorized_and_plugged_in_then_started_per_5_11()
 {
    let mut csms = authorizing_remote_starts(Csms::booted());
    remote_start_r1(&mut csms);
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    let ev = EvSpec::default();
    csms.intent(MINUTE, Intent::PlugIn { connector: 1, ev });

    assert_eq!(csms.expect("StartTransaction")["idTag"], "R1");
}
