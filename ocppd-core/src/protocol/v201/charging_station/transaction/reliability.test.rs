use crate::api::event::LinkEvent;
use crate::api::io::Input;
use crate::fixtures::v201::reliability::{configure, events, plug_in, status, swipe};
use crate::fixtures::v201::station::Run;
use crate::protocol::v201::event::EventKind;
use serde_json::json;

fn two_attempts(run: &mut Run) {
    configure(run, "OCPPCommCtrlr.MessageAttempts[TransactionEvent]", "2");
    configure(
        run,
        "OCPPCommCtrlr.MessageAttemptInterval[TransactionEvent]",
        "10",
    );
    plug_in(run, 0);
    run.answer(0, json!({}));
}

#[test]
fn given_booted_station_when_ev_plugged_offline_then_transaction_event_queued_per_e04_fr_01() {
    let mut run = Run::accepted(86_400);
    run.input(1_000, Input::Disconnected);

    plug_in(&mut run, 2_000);

    assert!(events(&run).is_empty());
}

#[test]
fn given_transaction_started_offline_when_back_online_then_sent_flagged_offline_with_its_time_per_e04_fr_03()
 {
    let mut run = Run::accepted(86_400);
    run.input(1_000, Input::Disconnected);
    plug_in(&mut run, 2_000);

    run.input(3_000, Input::Connected);
    run.answer_all(3_000);

    let started = &events(&run)[0];
    assert_eq!(started["eventType"], "Started");
    assert_eq!(started["offline"], true);
    assert_eq!(
        started["timestamp"], "1970-01-01T00:00:02Z",
        "when it happened"
    );
}

#[test]
fn given_booted_station_when_ev_plugged_then_started_event_waits_behind_the_status() {
    let mut run = Run::accepted(86_400);

    plug_in(&mut run, 1_000);

    assert_eq!(run.last_call().1, "StatusNotification");
}

#[test]
fn given_events_queued_when_connection_drops_and_returns_then_sent_flagged_offline_per_e11_fr_07() {
    let mut run = Run::accepted(86_400);
    plug_in(&mut run, 1_000);

    run.input(2_000, Input::Disconnected);
    run.input(3_000, Input::Connected);
    run.answer_all(3_000);

    assert_eq!(events(&run)[0]["offline"], true);
}

#[test]
fn given_offline_station_when_id_token_presented_then_refused() {
    let mut run = Run::accepted(86_400);
    run.input(1_000, Input::Disconnected);

    swipe(&mut run, 2_000, "04A2B3C4");

    assert!(matches!(
        run.events.last(),
        Some(EventKind::Link(LinkEvent::IntentRejected { .. }))
    ));
}

#[test]
fn given_two_attempts_when_transaction_event_refused_then_retried_after_the_interval_per_e13_fr_03()
{
    let mut run = Run::accepted(86_400);
    two_attempts(&mut run);

    run.fail(0);

    assert_eq!(
        run.station.next_deadline(),
        Some(10_000),
        "interval × 1, E13.FR.03"
    );
}

#[test]
fn given_refused_transaction_event_when_the_interval_elapses_then_sent_again_per_e13_fr_02() {
    let mut run = Run::accepted(86_400);
    two_attempts(&mut run);
    run.fail(0);

    run.input(10_000, Input::Tick);

    assert_eq!(events(&run).len(), 2);
}

#[test]
fn given_last_attempt_sent_when_refused_again_then_abandoned_per_e13_fr_04() {
    let mut run = Run::accepted(86_400);
    two_attempts(&mut run);
    run.fail(0);
    run.input(10_000, Input::Tick);

    run.fail(10_000);

    assert!(
        run.events
            .contains(&EventKind::Link(LinkEvent::CallAbandoned {
                action: "TransactionEvent".into()
            }))
    );
}

#[test]
fn given_no_transaction_when_status_asked_without_id_then_no_messages_queued_per_e14_fr_06_and_fr_08()
 {
    let mut run = Run::accepted(86_400);

    let reply = status(&mut run, json!({}));

    assert_eq!(reply, json!({"messagesInQueue": false}));
}

#[test]
fn given_queued_transaction_event_when_status_asked_without_id_then_messages_queued_per_e14_fr_07()
{
    let mut run = Run::accepted(86_400);
    plug_in(&mut run, 0);

    let reply = status(&mut run, json!({}));

    assert_eq!(reply, json!({"messagesInQueue": true}));
}

#[test]
fn given_ongoing_transaction_with_nothing_queued_when_status_asked_then_ongoing_per_e14_fr_02_and_fr_05()
 {
    let mut run = Run::accepted(86_400);
    plug_in(&mut run, 0);
    run.answer_all(0);
    let id = events(&run)[0]["transactionInfo"]["transactionId"].clone();

    let reply = status(&mut run, json!({"transactionId": id}));

    assert_eq!(
        reply,
        json!({"ongoingIndicator": true, "messagesInQueue": false})
    );
}

#[test]
fn given_booted_station_when_status_asked_for_unknown_id_then_not_ongoing_per_e14_fr_01_and_tc_e_28()
 {
    let mut run = Run::accepted(86_400);

    let reply = status(&mut run, json!({"transactionId": "nope"}));

    assert_eq!(
        reply,
        json!({"ongoingIndicator": false, "messagesInQueue": false})
    );
}
