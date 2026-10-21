use super::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    items: Vec<Inner>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    name: String,
}

fn violation(payload: Value) -> (Violation, String) {
    let invalid = decode::<Outer>(payload).unwrap_err();
    (invalid.violation, invalid.reason)
}

#[test]
fn given_payload_matching_the_type_when_decoded_then_typed() {
    let payload = json!({"count": 1, "items": [{"name": "a"}]});

    let decoded = decode::<Outer>(payload);

    let expected = Outer {
        count: 1,
        note: None,
        items: vec![Inner { name: "a".into() }],
    };
    assert_eq!(decoded, Ok(expected));
}

#[test]
fn given_invalid_payloads_when_decoded_then_each_gets_its_ocpp_j_violation() {
    let cases = [
        (json!([1]), Violation::Formation),
        (json!({"items": []}), Violation::Occurrence),
        (json!({"count": "one", "items": []}), Violation::Type),
        (json!({"count": -1, "items": []}), Violation::Property),
        (
            json!({"count": 1, "note": null, "items": []}),
            Violation::Type,
        ),
    ];

    for (payload, expected) in cases {
        let found = violation(payload.clone()).0;

        assert_eq!(found, expected, "{payload}");
    }
}

#[test]
fn given_unknown_top_level_field_when_decoded_then_formation_naming_it() {
    let top = json!({"count": 1, "items": [], "extra": true});

    let found = violation(top);

    assert_eq!(found, (Violation::Formation, "unknown field extra".into()));
}

#[test]
fn given_unknown_nested_field_when_decoded_then_formation_naming_its_path() {
    let nested = json!({"count": 1, "items": [{"name": "a"}, {"name": "b", "x": 1}]});

    let found = violation(nested);

    assert_eq!(
        found,
        (Violation::Formation, "unknown field items[1].x".into())
    );
}
