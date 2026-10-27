use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use serde_json::json;

fn actions(csms: &Csms) -> Vec<String> {
    csms.sent.iter().map(|(action, _)| action.clone()).collect()
}

fn offline_while_charging() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.disconnect(30_000);
    csms.sent.clear();
    csms
}

fn stopped_offline() -> Csms {
    let mut csms = offline_while_charging();
    csms.input(3 * MINUTE + 30_000, Input::Tick);
    csms.intent(3 * MINUTE + 30_000, Intent::Unplug { connector: 1 });
    csms
}

fn replayed() -> Csms {
    let mut csms = stopped_offline();
    csms.reconnect(4 * MINUTE);
    csms
}

fn swiped() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(
        0,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
    csms.intent(0, authorize("TAG1"));
    csms
}

fn start_in_flight() -> (Csms, serde_json::Value) {
    let mut csms = swiped();
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));
    let first = csms.expect("StartTransaction");
    csms.disconnect(1_000);
    (csms, first)
}

fn authorize_dropped() -> Csms {
    let mut csms = swiped();
    csms.disconnect(1_000);
    csms.sent.clear();
    csms.reconnect(10_000);
    csms
}

#[test]
fn given_offline_while_charging_when_samples_due_and_ev_unplugged_then_nothing_sent() {
    let mut csms = offline_while_charging();

    csms.input(3 * MINUTE + 30_000, Input::Tick);
    csms.intent(3 * MINUTE + 30_000, Intent::Unplug { connector: 1 });

    assert!(csms.sent.is_empty(), "nothing leaves while offline");
}

#[test]
fn given_transaction_messages_queued_offline_when_reconnected_then_replayed_in_order_without_a_boot()
 {
    let mut csms = stopped_offline();

    csms.reconnect(4 * MINUTE);

    assert_eq!(
        actions(&csms),
        vec![
            "MeterValues",
            "MeterValues",
            "MeterValues",
            "StopTransaction",
            "StatusNotification",
            "StatusNotification",
        ]
    );
}

#[test]
fn given_replayed_meter_values_when_read_then_stamped_when_sampled() {
    let csms = replayed();

    let timestamps: Vec<_> = csms
        .sent("MeterValues")
        .iter()
        .map(|sent| sent["meterValue"][0]["timestamp"].clone())
        .collect();

    assert_eq!(
        timestamps,
        vec![
            json!("1970-01-01T00:01:00Z"),
            json!("1970-01-01T00:02:00Z"),
            json!("1970-01-01T00:03:00Z"),
        ]
    );
}

#[test]
fn given_replayed_stop_transaction_when_read_then_stamped_at_the_unplug_with_its_register() {
    let csms = replayed();

    let stop = &csms.sent("StopTransaction")[0];

    assert_eq!(stop["reason"], "EVDisconnected");
    assert_eq!(stop["timestamp"], "1970-01-01T00:03:30Z");
    assert_eq!(stop["meterStop"], 642);
}

#[test]
fn given_start_transaction_in_flight_offline_when_reconnected_then_sent_again() {
    let (mut csms, first) = start_in_flight();

    csms.reconnect(10_000);

    assert_eq!(csms.expect("StartTransaction"), first);
}

#[test]
fn given_start_transaction_resent_when_answered_then_transaction_started() {
    let (mut csms, _) = start_in_flight();
    csms.reconnect(10_000);

    csms.answer(
        10_000,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 8}),
    );

    assert!(csms.events.contains(&EventKind::TransactionStarted {
        connector: 1,
        transaction_id: 8
    }));
}

#[test]
fn given_authorize_in_flight_when_disconnected_and_reconnected_then_dropped_and_still_preparing() {
    let mut csms = swiped();
    csms.disconnect(1_000);
    csms.sent.clear();

    csms.reconnect(10_000);

    assert!(!actions(&csms).contains(&"Authorize".to_string()));
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Preparing));
}

#[test]
fn given_dropped_authorize_when_tag_swiped_again_then_authorize_sent() {
    let mut csms = authorize_dropped();

    csms.intent(11_000, authorize("TAG1"));

    assert_eq!(csms.expect("Authorize"), json!({"idTag": "TAG1"}));
}
