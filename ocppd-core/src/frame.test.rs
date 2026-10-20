use super::*;

fn round_trip(frame: &Frame) -> Result<Frame, FrameError> {
    parse(&serialize(frame))
}

#[test]
fn given_call_when_serialized_and_parsed_then_unchanged() {
    let call = Frame::Call {
        id: "1".into(),
        action: "Heartbeat".into(),
        payload: json!({}),
    };

    let parsed = round_trip(&call);

    assert_eq!(parsed, Ok(call));
}

#[test]
fn given_call_result_when_serialized_and_parsed_then_unchanged() {
    let result = Frame::CallResult {
        id: "1".into(),
        payload: json!({"currentTime": "2026-10-19T00:00:00Z"}),
    };

    let parsed = round_trip(&result);

    assert_eq!(parsed, Ok(result));
}

#[test]
fn given_call_error_when_serialized_and_parsed_then_unchanged() {
    let error = Frame::CallError {
        id: "1".into(),
        code: "NotImplemented".into(),
        description: "".into(),
        details: json!({}),
    };

    let parsed = round_trip(&error);

    assert_eq!(parsed, Ok(error));
}

#[test]
fn given_wire_text_of_a_call_when_parsed_then_its_id_action_and_payload() {
    let text = r#"[2,"19223201","BootNotification",{"chargePointVendor":"x"}]"#;

    let frame = parse(text);

    assert_eq!(
        frame,
        Ok(Frame::Call {
            id: "19223201".into(),
            action: "BootNotification".into(),
            payload: json!({"chargePointVendor": "x"}),
        })
    );
}

#[test]
fn given_malformed_frames_when_parsed_then_each_gets_its_error() {
    let cases = [
        ("[2,", FrameError::NotJson),
        (r#"{"a":1}"#, FrameError::NotArray),
        ("[]", FrameError::UnknownType),
        (r#"[5,"1",{}]"#, FrameError::UnknownType),
        (r#"["2","1","Heartbeat",{}]"#, FrameError::UnknownType),
        (r#"[2,"1",{}]"#, FrameError::WrongLength),
        (r#"[3,"1",{},{}]"#, FrameError::WrongLength),
        (r#"[2,1,"Heartbeat",{}]"#, FrameError::BadField("id")),
        (r#"[2,"1",7,{}]"#, FrameError::BadField("action")),
        (r#"[4,"1","x","y",null]"#, FrameError::BadField("details")),
    ];

    for (text, error) in cases {
        let parsed = parse(text);

        assert_eq!(parsed, Err(error), "input: {text}");
    }
}
