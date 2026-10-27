use super::*;
use serde_json::{Value, json};

#[test]
fn given_a_rejected_command_for_a_station_when_serialized_then_an_event_naming_both() {
    let rejection = Rejection::new(Some("stop"), Some("CP9"), "unknown station");

    let line: Value = serde_json::to_value(&rejection).unwrap();

    assert_eq!(line["kind"], "command_rejected");
    assert_eq!(line["command"], "stop");
    assert_eq!(line["station"], "CP9");
    assert_eq!(line["reason"], "unknown station");
    assert!(line["at"].as_u64().unwrap() > 0);
}

#[test]
fn given_a_line_that_is_not_a_command_when_serialized_then_no_command_or_station() {
    let rejection = Rejection::new(None, None, "expected value at line 1 column 1");

    let mut line: Value = serde_json::to_value(&rejection).unwrap();
    line.as_object_mut().unwrap().remove("at");

    assert_eq!(
        line,
        json!({"kind": "command_rejected", "reason": "expected value at line 1 column 1"})
    );
}
