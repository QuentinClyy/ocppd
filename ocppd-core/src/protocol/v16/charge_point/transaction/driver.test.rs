use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::core::status::ChargePointStatus;
use crate::protocol::v16::messages::core::transaction::Reason;
use crate::protocol::v16::messages::datatypes::id_tag::AuthorizationStatus;
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
    vec![EventKind::Link(LinkEvent::IntentRejected {
        connector,
        reason: reason.into(),
    })]
}

fn plugged() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, plug_in());
    csms
}

fn offline() -> Csms {
    let mut offline = Csms::booted();
    offline.input(0, Input::Disconnected);
    offline.events.clear();
    offline
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
fn given_charging_ev_when_unplugged_then_stopped_as_ev_disconnected_and_available() {
    let mut csms = Csms::charging(0, EvSpec::default());

    csms.intent(5 * MINUTE, Intent::Unplug { connector: 1 });

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["reason"], "EVDisconnected");
    assert!(csms.events.contains(&EventKind::TransactionStopped {
        connector: 1,
        transaction_id: Some(42),
        reason: Reason::EvDisconnected
    }));
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Available));
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

#[test]
fn given_booted_charge_point_when_tag_over_20_characters_swiped_then_rejected() {
    let mut csms = Csms::booted();
    let intent = authorize("an id tag over twenty");
    let connector = intent.connector();

    let events = rejected(&mut csms, intent);

    assert_eq!(
        events,
        intent_rejected(
            connector,
            "\"an id tag over twenty\" is longer than 20 characters"
        )
    );
}

#[test]
fn given_offline_charge_point_when_ev_plugged_then_nothing_rejected() {
    let mut offline = offline();

    offline.intent(0, plug_in());

    assert!(offline.events.is_empty());
}

#[test]
fn given_offline_with_plugged_ev_when_unlisted_tag_swiped_then_rejected_as_offline() {
    let mut offline = offline();
    offline.intent(0, plug_in());

    offline.intent(0, authorize("TAG1"));

    assert_eq!(offline.events, intent_rejected(Some(1), "offline"));
}

fn power_returns(csms: &mut Csms, now: u64) {
    csms.input(now, Input::Disconnected);
    csms.input(now + MINUTE, Input::Connected);
    csms.answer(
        now + MINUTE,
        json!({"status": "Accepted", "currentTime": "1970-01-01T00:02:00Z", "interval": 300}),
    );
}

#[test]
fn given_charging_ev_when_power_cut_then_stopped_as_power_loss_and_down_once_drained_per_tc_032_1()
{
    let mut csms = Csms::charging(0, EvSpec::default());

    csms.intent(MINUTE, Intent::PowerCut);

    assert!(csms.closed, "down once the outbox is drained");
    assert_eq!(csms.sent("StopTransaction")[0]["reason"], "PowerLoss");
}

#[test]
fn given_power_cut_while_charging_when_power_returns_then_finishing_reported_before_and_booted_again_with_the_ev_plugged_per_tc_032_1()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.intent(MINUTE, Intent::PowerCut);

    power_returns(&mut csms, MINUTE);

    let order: Vec<(String, String)> = csms
        .sent
        .iter()
        .map(|(action, payload)| {
            (
                action.clone(),
                payload["status"].as_str().unwrap_or("").to_owned(),
            )
        })
        .collect();
    let finishing = order
        .iter()
        .position(|(action, status)| action == "StatusNotification" && status == "Finishing");
    let boot = order
        .iter()
        .position(|(action, _)| action == "BootNotification");
    assert!(
        finishing < boot && finishing.is_some(),
        "Finishing before going down: {order:?}"
    );
    assert_eq!(csms.sent("BootNotification").len(), 1, "booted again");
    let after_boot = *csms.statuses().last().unwrap();
    assert!(
        matches!(
            after_boot,
            ChargePointStatus::Finishing | ChargePointStatus::Preparing
        ),
        "EV still plugged: {after_boot:?}"
    );
}

#[test]
fn given_inoperative_station_cut_from_power_when_power_returns_then_still_unavailable_per_tc_034() {
    let mut csms = Csms::booted();
    let inoperative = json!({"connectorId": 0, "type": "Inoperative"});
    csms.call(0, "ChangeAvailability", inoperative);
    csms.intent(MINUTE, Intent::PowerCut);

    power_returns(&mut csms, MINUTE);

    let last = csms.sent("StatusNotification").pop().unwrap();
    assert_eq!(last["status"], "Unavailable");
}
