use crate::api::event::EventKind;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::fixtures::v16::replies::status;
use crate::v16::messages::core::status::ChargePointStatus;
use crate::v16::messages::datatypes::id_tag::AuthorizationStatus;
use serde_json::{Value, json};

fn swipe(connector: Option<u32>) -> Intent {
    Intent::Authorize {
        connector,
        id_tag: "TAG1".into(),
    }
}

fn plug_in(connector: u32) -> Intent {
    Intent::PlugIn {
        connector,
        ev: EvSpec::default(),
    }
}

fn accepted() -> Value {
    json!({"idTagInfo": {"status": "Accepted"}})
}

fn swiped() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, swipe(Some(1)));
    csms
}

fn held() -> Csms {
    let mut csms = swiped();
    csms.answer(0, accepted());
    csms
}

fn held_without_connector() -> Csms {
    let mut csms = Csms::booted_with(2);
    csms.intent(0, swipe(None));
    csms.answer(0, accepted());
    csms
}

#[test]
fn given_nothing_plugged_when_tag_swiped_on_connector_1_then_authorize_sent_and_preparing() {
    let mut csms = Csms::booted();

    csms.intent(0, swipe(Some(1)));

    assert_eq!(csms.expect("Authorize"), json!({"idTag": "TAG1"}));
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Preparing]);
}

#[test]
fn given_swiped_tag_when_authorize_accepted_then_held_until_the_connection_time_out() {
    let mut csms = swiped();

    csms.answer(0, accepted());

    assert!(csms.events.contains(&EventKind::Authorization {
        connector: Some(1),
        id_tag: "TAG1".into(),
        status: AuthorizationStatus::Accepted
    }));
    assert_eq!(csms.charge_point.next_deadline(), Some(MINUTE));
}

#[test]
fn given_held_authorization_when_ev_plugged_then_start_transaction_for_the_tag() {
    let mut csms = held();

    csms.intent(10_000, plug_in(1));

    let start = csms.expect("StartTransaction");
    assert_eq!(start["idTag"], "TAG1");
    assert_eq!(start["connectorId"], 1);
}

#[test]
fn given_two_connectors_when_tag_swiped_without_connector_and_accepted_then_no_connector_preparing()
{
    let mut csms = Csms::booted_with(2);
    csms.intent(0, swipe(None));

    csms.answer(0, accepted());

    assert!(csms.statuses().is_empty(), "no connector is waiting yet");
}

#[test]
fn given_authorization_without_connector_when_ev_plugged_on_2_then_started_on_2() {
    let mut csms = held_without_connector();

    csms.intent(5_000, plug_in(2));

    assert_eq!(csms.expect("StartTransaction")["connectorId"], 2);
}

fn held_for_30_seconds() -> Csms {
    let mut csms = Csms::booted();
    csms.configure(0, "ConnectionTimeOut", "30");
    csms.intent(0, swipe(Some(1)));
    csms.answer(0, accepted());
    csms
}

fn rejected_swipe() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, swipe(Some(1)));
    csms.answer(0, json!({"idTagInfo": {"status": "Invalid"}}));
    csms
}

fn remote_start(csms: &mut Csms) -> Value {
    let request = json!({"connectorId": 1, "idTag": "R1"});
    status(csms.call(0, "RemoteStartTransaction", request))
}

#[test]
fn given_authorization_held_for_30_seconds_when_they_pass_then_expired_and_available() {
    let mut csms = held_for_30_seconds();

    csms.input(30_000, Input::Tick);

    assert!(csms.events.contains(&EventKind::AuthorizationExpired {
        id_tag: "TAG1".into()
    }));
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Available));
}

#[test]
fn given_expired_authorization_when_ev_plugged_then_no_transaction() {
    let mut csms = held_for_30_seconds();
    csms.input(30_000, Input::Tick);

    csms.intent(31_000, plug_in(1));

    assert!(csms.sent("StartTransaction").is_empty());
}

#[test]
fn given_swiped_tag_when_authorize_invalid_then_nothing_held() {
    let mut csms = Csms::booted();
    csms.intent(0, swipe(Some(1)));

    csms.answer(0, json!({"idTagInfo": {"status": "Invalid"}}));

    assert_eq!(csms.charge_point.next_deadline(), Some(86_400_000));
}

#[test]
fn given_rejected_swipe_when_ev_plugged_then_no_transaction() {
    let mut csms = rejected_swipe();

    csms.intent(1_000, plug_in(1));

    assert!(csms.sent("StartTransaction").is_empty());
}

#[test]
fn given_two_evs_waiting_when_tag_swiped_without_connector_then_rejected_as_ambiguous() {
    let mut csms = Csms::booted_with(2);
    csms.intent(0, plug_in(1));
    csms.intent(0, plug_in(2));
    csms.events.clear();

    csms.intent(0, swipe(None));

    let expected = EventKind::IntentRejected {
        connector: None,
        reason: "ambiguous connector".into(),
    };
    assert_eq!(csms.events, vec![expected]);
}

#[test]
fn given_held_authorization_when_another_tag_swiped_then_rejected_as_pending() {
    let mut csms = Csms::booted();
    csms.intent(0, swipe(Some(1)));
    csms.answer(0, accepted());
    csms.events.clear();

    csms.intent(1_000, swipe(None));

    let expected = EventKind::IntentRejected {
        connector: None,
        reason: "authorization pending".into(),
    };
    assert_eq!(csms.events, vec![expected]);
}

#[test]
fn given_nothing_plugged_when_remote_start_then_accepted_without_authorize_and_waits_a_minute_preparing()
 {
    let mut csms = Csms::booted();

    let status = remote_start(&mut csms);

    assert_eq!(status, "Accepted");
    assert!(csms.sent("Authorize").is_empty());
    assert!(csms.events.contains(&EventKind::RemoteStart {
        connector: Some(1),
        id_tag: "R1".into(),
        expires: Some(MINUTE)
    }));
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Preparing]);
}
