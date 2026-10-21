use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::v16::event::EventKind;
use serde_json::{Value, json};

fn offline_with_list() -> Csms {
    let mut csms = Csms::booted();
    assert_eq!(
        csms.configure(0, "LocalAuthorizeOffline", "true"),
        "Accepted"
    );
    let list = json!({
        "listVersion": 1,
        "updateType": "Full",
        "localAuthorizationList": [{"idTag": "TAG1", "idTagInfo": {"status": "Accepted"}}],
    });
    csms.call(0, "SendLocalList", list);
    csms.disconnect(0);
    csms.events.clear();
    csms
}

fn start(csms: &mut Csms, now: u64) {
    let ev = EvSpec::default();
    csms.intent(now, Intent::PlugIn { connector: 1, ev });
    csms.intent(now, authorize("TAG1"));
}

fn unplug(csms: &mut Csms, now: u64) {
    csms.intent(now, Intent::Unplug { connector: 1 });
}

fn started(transaction_id: i32) -> Value {
    json!({"idTagInfo": {"status": "Accepted"}, "transactionId": transaction_id})
}

fn transaction_ids(csms: &Csms, action: &str) -> Vec<Value> {
    let sent = csms.sent(action);
    sent.iter()
        .map(|sent| sent["transactionId"].clone())
        .collect()
}

#[test]
fn given_listed_id_tag_offline_when_it_charges_then_samples_are_taken_offline() {
    let mut csms = offline_with_list();
    start(&mut csms, 0);

    csms.input(2 * MINUTE, Input::Tick);

    let samples = csms.events.iter();
    let samples = samples.filter(|kind| matches!(kind, EventKind::MeterSample { .. }));
    assert_eq!(samples.count(), 2, "it charges while offline");
}

#[test]
fn given_offline_transaction_when_reconnected_then_start_transaction_sent_with_its_time() {
    let mut csms = offline_with_list();
    start(&mut csms, 0);
    csms.input(2 * MINUTE, Input::Tick);

    csms.reconnect(3 * MINUTE);

    let request = csms.expect("StartTransaction");
    assert_eq!(request["timestamp"], "1970-01-01T00:00:00Z");
}

#[test]
fn given_offline_transaction_sent_when_its_start_answered_then_queued_meter_values_get_the_transaction_id()
 {
    let mut csms = offline_with_list();
    start(&mut csms, 0);
    csms.input(2 * MINUTE, Input::Tick);
    csms.reconnect(3 * MINUTE);

    csms.answer(3 * MINUTE, started(9));

    assert_eq!(
        transaction_ids(&csms, "MeterValues"),
        vec![json!(9), json!(9)]
    );
    assert!(csms.events.contains(&EventKind::TransactionStarted {
        connector: 1,
        transaction_id: 9
    }));
}

#[test]
fn given_transaction_started_and_stopped_offline_when_its_start_answered_then_one_stop_with_the_transaction_id()
 {
    let mut csms = offline_with_list();
    start(&mut csms, 0);
    unplug(&mut csms, 30_000);
    csms.reconnect(MINUTE);

    csms.answer(MINUTE, started(9));

    assert_eq!(transaction_ids(&csms, "StopTransaction"), vec![json!(9)]);
}

#[test]
fn given_two_offline_sessions_when_their_starts_answered_then_each_stop_gets_its_own_id() {
    let mut csms = offline_with_list();
    start(&mut csms, 0);
    unplug(&mut csms, 10_000);
    start(&mut csms, 20_000);
    unplug(&mut csms, 30_000);
    csms.reconnect(MINUTE);

    csms.answer(MINUTE, started(1));
    csms.answer(MINUTE, started(2));

    let stops = transaction_ids(&csms, "StopTransaction");
    assert_eq!(stops, vec![json!(1), json!(2)]);
}

#[test]
fn given_offline_transaction_when_its_id_tag_is_refused_on_reconnect_then_stopped_as_deauthorized()
{
    let mut csms = offline_with_list();
    start(&mut csms, 0);
    csms.reconnect(MINUTE);
    let refused = json!({"idTagInfo": {"status": "Invalid"}, "transactionId": 9});

    csms.answer(MINUTE, refused);

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["transactionId"], 9);
    assert_eq!(stop["reason"], "DeAuthorized");
}

#[test]
fn given_offline_with_list_when_an_unlisted_id_tag_presented_then_refused_as_offline() {
    let mut csms = offline_with_list();
    let ev = EvSpec::default();
    csms.intent(0, Intent::PlugIn { connector: 1, ev });

    csms.intent(0, authorize("TAG2"));

    assert_eq!(
        csms.events,
        vec![EventKind::Link(LinkEvent::IntentRejected {
            connector: Some(1),
            reason: "offline".into(),
        })]
    );
}

#[test]
fn given_one_attempt_and_an_offline_session_when_its_start_fails_then_both_start_and_stop_abandoned()
 {
    let mut csms = offline_with_list();
    csms.configure(0, "TransactionMessageAttempts", "1");
    start(&mut csms, 0);
    unplug(&mut csms, 30_000);
    csms.reconnect(MINUTE);
    csms.expect("StartTransaction");

    csms.fail(MINUTE);

    assert!(csms.sent("StopTransaction").is_empty());
    let abandoned = |action: &str| {
        EventKind::Link(LinkEvent::CallAbandoned {
            action: action.into(),
        })
    };
    assert!(csms.events.contains(&abandoned("StartTransaction")));
    assert!(csms.events.contains(&abandoned("StopTransaction")));
}
