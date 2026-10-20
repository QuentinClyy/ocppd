use crate::api::event::EventKind;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::v16::messages::core::status::ChargePointStatus;

fn stopping() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.answered_by_test.push("StopTransaction");
    csms.intent(MINUTE, Intent::Stop { connector: 1 });
    csms
}

fn retried(csms: &Csms) -> Vec<(u32, u64)> {
    csms.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::CallRetried {
                attempt, retry_at, ..
            } => Some((*attempt, *retry_at)),
            _ => None,
        })
        .collect()
}

fn failed_once() -> Csms {
    let mut csms = stopping();
    csms.fail(MINUTE);
    csms
}

fn failed_twice() -> Csms {
    let mut csms = failed_once();
    csms.input(2 * MINUTE, Input::Tick);
    csms.fail(2 * MINUTE);
    csms
}

fn two_attempts_without_wait() -> Csms {
    let mut csms = stopping();
    csms.configure(MINUTE, "TransactionMessageAttempts", "2");
    csms.configure(MINUTE, "TransactionMessageRetryInterval", "0");
    csms
}

#[test]
fn given_stop_transaction_sent_when_it_fails_then_retry_due_in_a_minute() {
    let mut csms = stopping();

    csms.fail(MINUTE);

    assert_eq!(retried(&csms), vec![(1, 2 * MINUTE)]);
    assert_eq!(csms.charge_point.next_deadline(), Some(2 * MINUTE));
}

#[test]
fn given_failed_stop_transaction_when_just_before_the_retry_time_then_not_resent() {
    let mut csms = failed_once();

    csms.input(2 * MINUTE - 1, Input::Tick);

    assert_eq!(
        csms.sent("StopTransaction").len(),
        1,
        "not before the retry time"
    );
}

#[test]
fn given_failed_stop_transaction_when_the_retry_time_comes_then_resent_unchanged() {
    let mut csms = stopping();
    let first = csms.expect("StopTransaction");
    csms.fail(MINUTE);

    csms.input(2 * MINUTE, Input::Tick);

    assert_eq!(csms.expect("StopTransaction"), first);
}

#[test]
fn given_resent_stop_transaction_when_it_fails_again_then_the_delay_doubles() {
    let mut csms = failed_once();
    csms.input(2 * MINUTE, Input::Tick);

    csms.fail(2 * MINUTE);

    assert_eq!(retried(&csms).last(), Some(&(2, 4 * MINUTE)));
}

#[test]
fn given_two_failures_when_the_third_attempt_is_answered_then_finishing() {
    let mut csms = failed_twice();
    csms.input(4 * MINUTE, Input::Tick);

    csms.answer(4 * MINUTE, serde_json::json!({}));

    assert_eq!(csms.sent("StopTransaction").len(), 3);
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Finishing));
}

#[test]
fn given_two_attempts_when_the_last_one_fails_then_abandoned() {
    let mut csms = two_attempts_without_wait();
    csms.fail(MINUTE);
    csms.input(MINUTE, Input::Tick);

    csms.fail(MINUTE);

    assert!(csms.events.contains(&EventKind::CallAbandoned {
        action: "StopTransaction".into()
    }));
    assert_eq!(csms.sent("StopTransaction").len(), 2);
}

#[test]
fn given_status_notification_sent_when_it_fails_then_not_retried() {
    let mut csms = Csms::booted();
    csms.answered_by_test.push("StatusNotification");
    csms.intent(
        0,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
    csms.expect("StatusNotification");

    csms.fail(0);

    assert!(retried(&csms).is_empty());
}

#[test]
fn given_stop_transaction_waiting_to_retry_when_heartbeat_due_then_heartbeat_sent_alone() {
    let mut csms = failed_once();
    csms.configure(MINUTE, "HeartbeatInterval", "10");

    csms.input(MINUTE + 10_000, Input::Tick);

    assert_eq!(csms.sent("Heartbeat").len(), 1);
    assert_eq!(csms.sent("StopTransaction").len(), 1);
}

#[test]
fn given_two_attempts_and_a_disconnect_when_the_resent_one_fails_then_not_abandoned_and_sent_again()
{
    let mut csms = two_attempts_without_wait();
    csms.disconnect(MINUTE);
    csms.reconnect(MINUTE);
    csms.expect("StopTransaction");

    csms.fail(MINUTE);

    assert!(!csms.events.contains(&EventKind::CallAbandoned {
        action: "StopTransaction".into()
    }));
    csms.expect("StopTransaction");
}
