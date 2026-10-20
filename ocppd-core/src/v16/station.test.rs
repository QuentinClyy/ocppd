use crate::event::{EventKind, Input, Output};
use crate::fixtures::v16::raw::{boot_response, calls, charge_point, kinds, only_call, result};
use crate::frame::Frame;
use crate::v16::station::Station;
use serde_json::json;

fn booted_and_reported(now: u64) -> (Station, Vec<Output>) {
    let mut charge_point = charge_point();
    let (boot_id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let outputs = charge_point.handle(now, result(&boot_id, boot_response("Accepted", 300)));
    (charge_point, outputs)
}

fn drained(charge_point: &mut Station, now: u64, mut outputs: Vec<Output>) {
    while let Some((id, _, _)) = calls(&outputs).pop() {
        outputs = charge_point.handle(now, result(&id, json!({})));
    }
}

fn idle() -> Station {
    let (mut charge_point, outputs) = booted_and_reported(1_000);
    drained(&mut charge_point, 1_000, outputs);
    charge_point
}

#[test]
fn given_idle_charge_point_when_just_before_the_first_heartbeat_then_nothing_sent() {
    let mut charge_point = idle();
    let first = charge_point.next_deadline().unwrap();

    let outputs = charge_point.handle(first - 1, Input::Tick);

    assert!(
        (1_000..=301_000).contains(&first),
        "first heartbeat at {first}"
    );
    assert!(calls(&outputs).is_empty());
}

#[test]
fn given_idle_charge_point_when_the_first_heartbeat_is_due_then_empty_heartbeat_sent() {
    let mut charge_point = idle();
    let first = charge_point.next_deadline().unwrap();

    let outputs = charge_point.handle(first, Input::Tick);

    let (_, action, payload) = only_call(&outputs);
    assert_eq!(action, "Heartbeat");
    assert_eq!(payload, json!({}));
}

#[test]
fn given_sent_heartbeat_when_answered_then_next_one_an_interval_later() {
    let mut charge_point = idle();
    let first = charge_point.next_deadline().unwrap();
    let (heartbeat_id, _, _) = only_call(&charge_point.handle(first, Input::Tick));

    charge_point.handle(
        first + 500,
        result(&heartbeat_id, json!({"currentTime": "x"})),
    );

    assert_eq!(charge_point.next_deadline(), Some(first + 300_000));
}

fn answered_boot(status: &str) -> (Station, String, Vec<Output>) {
    let mut charge_point = charge_point();
    let (first_id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let outputs = charge_point.handle(1_000, result(&first_id, boot_response(status, 60)));
    (charge_point, first_id, outputs)
}

#[test]
fn given_sent_boot_when_rejected_or_pending_then_nothing_sent_and_retry_after_the_interval() {
    for status in ["Rejected", "Pending"] {
        let (charge_point, _, outputs) = answered_boot(status);

        let deadline = charge_point.next_deadline();

        assert!(calls(&outputs).is_empty(), "{status}");
        assert_eq!(deadline, Some(61_000), "{status}");
    }
}

#[test]
fn given_rejected_or_pending_boot_when_the_interval_passes_then_new_boot_notification() {
    for status in ["Rejected", "Pending"] {
        let (mut charge_point, first_id, _) = answered_boot(status);

        let outputs = charge_point.handle(61_000, Input::Tick);

        let (retry_id, action, _) = only_call(&outputs);
        assert_eq!(action, "BootNotification", "{status}");
        assert_ne!(retry_id, first_id);
    }
}

#[test]
fn given_pending_boot_when_answer_with_an_unknown_id_arrives_then_reported_as_unknown_response() {
    let mut charge_point = charge_point();
    charge_point.handle(0, Input::Connected);

    let outputs = charge_point.handle(1_000, result("nope", json!({})));

    assert_eq!(
        kinds(&outputs),
        vec![EventKind::UnknownResponse { id: "nope".into() }]
    );
}

#[test]
fn given_pending_boot_when_csms_call_received_then_answered_right_away() {
    let mut charge_point = charge_point();
    charge_point.handle(0, Input::Connected);
    let call = json!([2, "abc", "GetVariables", {}]).to_string();

    let outputs = charge_point.handle(1_000, Input::Frame(call));

    assert!(matches!(
        outputs[..],
        [Output::Send(Frame::CallError { .. })]
    ));
}

#[test]
fn given_new_charge_point_when_frame_is_not_json_then_reported_as_rejected() {
    let mut charge_point = charge_point();

    let outputs = charge_point.handle(0, Input::Frame("nope".into()));

    assert_eq!(
        kinds(&outputs),
        vec![EventKind::FrameRejected {
            reason: "frame is not valid JSON".into()
        }]
    );
}
