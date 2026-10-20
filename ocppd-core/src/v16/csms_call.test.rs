use crate::event::EventKind;
use crate::event::{Input, Output};
use crate::fixtures::v16::raw::{accepted_charge_point, charge_point, kinds, result};
use crate::frame::Frame;
use serde_json::json;

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
fn given_accepted_charge_point_when_2_0_1_call_received_then_not_implemented_error() {
    let mut charge_point = accepted_charge_point(1_000, 300);
    let call = json!([2, "abc", "GetVariables", {}]).to_string();

    let outputs = charge_point.handle(2_000, Input::Frame(call));

    assert_eq!(
        outputs,
        vec![Output::Send(Frame::CallError {
            id: "abc".into(),
            code: "NotImplemented".into(),
            description: "GetVariables is not implemented".into(),
            details: json!({}),
        })]
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

#[test]
fn given_accepted_charge_point_when_disconnected_then_reported_and_no_deadline() {
    let mut charge_point = accepted_charge_point(1_000, 300);

    let outputs = charge_point.handle(2_000, Input::Disconnected);

    assert_eq!(kinds(&outputs), vec![EventKind::Disconnected]);
    assert_eq!(charge_point.next_deadline(), None);
}
