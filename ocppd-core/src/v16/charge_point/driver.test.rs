use crate::api::event::EventKind;
use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, authorize};
use crate::v16::messages::core::status::ChargePointStatus;
use crate::v16::messages::datatypes::id_tag::AuthorizationStatus;
use serde_json::json;

fn plug_in() -> Intent {
    Intent::PlugIn {
        connector: 1,
        ev: EvSpec::default(),
    }
}

fn swiped() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, plug_in());
    csms.intent(0, authorize("TAG1"));
    csms
}

fn rejected(csms: &mut Csms, intent: Intent) -> Vec<EventKind> {
    csms.events.clear();
    csms.intent(0, intent);
    csms.events.clone()
}

fn intent_rejected(connector: Option<u32>, reason: &str) -> Vec<EventKind> {
    vec![EventKind::IntentRejected {
        connector,
        reason: reason.into(),
    }]
}

fn plugged() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, plug_in());
    csms
}

#[test]
fn given_swiped_tag_when_authorize_blocked_then_nothing_starts_and_still_preparing() {
    let mut csms = swiped();

    csms.answer(0, json!({"idTagInfo": {"status": "Blocked"}}));

    assert!(csms.events.contains(&EventKind::Authorization {
        connector: Some(1),
        id_tag: "TAG1".into(),
        status: AuthorizationStatus::Blocked
    }));
    assert!(csms.sent("StartTransaction").is_empty());
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Preparing]);
}

#[test]
fn given_start_transaction_sent_when_answered_invalid_then_stopped_as_deauthorized() {
    let mut csms = swiped();
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    csms.answer(
        0,
        json!({"idTagInfo": {"status": "Invalid"}, "transactionId": 7}),
    );

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["reason"], "DeAuthorized");
    assert_eq!(stop["meterStop"], 0);
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Finishing));
}

#[test]
fn given_one_connector_when_connectors_0_and_2_stopped_then_rejected_as_unknown() {
    for connector in [2, 0] {
        let mut csms = Csms::booted();

        let events = rejected(&mut csms, Intent::Stop { connector });

        assert_eq!(
            events,
            intent_rejected(Some(connector), "unknown connector")
        );
    }
}

#[test]
fn given_nothing_plugged_when_unplugged_then_rejected() {
    let mut csms = Csms::booted();

    let events = rejected(&mut csms, Intent::Unplug { connector: 1 });

    assert_eq!(events, intent_rejected(Some(1), "nothing plugged in"));
}

#[test]
fn given_no_transaction_when_stopped_then_rejected() {
    let mut csms = Csms::booted();

    let events = rejected(&mut csms, Intent::Stop { connector: 1 });

    assert_eq!(events, intent_rejected(Some(1), "no active transaction"));
}

#[test]
fn given_plugged_ev_when_plugged_again_then_rejected_and_only_one_status_sent() {
    let mut csms = plugged();

    let events = rejected(&mut csms, plug_in());

    assert_eq!(events, intent_rejected(Some(1), "already plugged in"));
    let actions: Vec<&str> = csms
        .sent
        .iter()
        .map(|(action, _)| action.as_str())
        .collect();
    assert_eq!(actions, vec!["StatusNotification"]);
}
