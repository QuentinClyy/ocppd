use super::*;

#[test]
fn given_ev_with_the_old_max_kw_field_when_plug_in_read_then_refused_naming_the_field() {
    let line = r#"{"command":"plug_in","connector":1,"ev":{"max_kw":7}}"#;

    let error = serde_json::from_str::<Intent>(line).unwrap_err();

    assert!(
        error.to_string().contains("unknown field `max_kw`"),
        "{error}"
    );
}

#[test]
fn given_ev_with_some_fields_when_plug_in_read_then_the_rest_are_defaults() {
    let line = r#"{"command":"plug_in","connector":1,"ev":{"dc_kw":270}}"#;

    let intent = serde_json::from_str::<Intent>(line).unwrap();

    let expected = EvSpec {
        dc_kw: 270.0,
        ..EvSpec::default()
    };
    assert_eq!(
        intent,
        Intent::PlugIn {
            connector: 1,
            ev: expected
        }
    );
}
