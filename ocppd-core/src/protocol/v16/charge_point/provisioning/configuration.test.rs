use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::protocol::v16::event::EventKind;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn result(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { id, payload } => {
            assert_eq!(id, "csms-1");
            payload
        }
        other => panic!("expected a result, got {other:?}"),
    }
}

fn value_of(csms: &mut Csms, key: &str) -> Value {
    let payload = result(csms.call(0, "GetConfiguration", json!({"key": [key]})));
    payload["configurationKey"][0]["value"].clone()
}

#[test]
fn given_booted_charge_point_when_every_key_asked_then_all_35_keys_without_unknown() {
    let mut csms = Csms::booted();

    let payload = result(csms.call(0, "GetConfiguration", json!({})));

    let keys = payload["configurationKey"].as_array().unwrap();
    assert_eq!(keys.len(), 35);
    assert!(keys.contains(&json!({"key": "NumberOfConnectors", "readonly": true, "value": "1"})));
    assert!(
        keys.contains(&json!({"key": "HeartbeatInterval", "readonly": false, "value": "86400"}))
    );
    assert!(payload.get("unknownKey").is_none());
}

#[test]
fn given_booted_charge_point_when_known_and_unknown_keys_asked_then_known_listed_and_unknown_reported()
 {
    let mut csms = Csms::booted();
    let request = json!({"key": ["meterValueSampleInterval", "Nope"]});

    let payload = result(csms.call(0, "GetConfiguration", request));

    assert_eq!(
        payload,
        json!({
            "configurationKey": [{"key": "MeterValueSampleInterval", "readonly": false, "value": "60"}],
            "unknownKey": ["Nope"]
        })
    );
}

#[test]
fn given_booted_charge_point_when_keys_changed_then_only_values_the_station_honors_accepted() {
    let mut csms = Csms::booted();
    let cases = [
        ("Nope", "1", "NotSupported"),
        ("NumberOfConnectors", "2", "Rejected"),
        ("HeartbeatInterval", "soon", "Rejected"),
        ("HeartbeatInterval", "-5", "Rejected"),
        ("StopTransactionOnEVSideDisconnect", "false", "Rejected"),
        ("StopTransactionOnEVSideDisconnect", "true", "Accepted"),
        ("MeterValuesSampledData", "Temperature", "Rejected"),
        ("MeterValuesSampledData", "Voltage,Voltage", "Rejected"),
        (
            "MeterValuesSampledData",
            "Voltage, Power.Active.Import",
            "Accepted",
        ),
        ("heartbeatinterval", "120", "Accepted"),
    ];

    let statuses: Vec<_> = cases
        .iter()
        .map(|(key, value, _)| csms.configure(0, key, value))
        .collect();

    for ((key, value, expected), status) in cases.iter().zip(&statuses) {
        assert_eq!(status, expected, "{key}={value}");
    }
    assert_eq!(value_of(&mut csms, "HeartbeatInterval"), "120");
    assert!(csms.events.contains(&EventKind::ConfigurationChanged {
        key: "heartbeatinterval".into(),
        value: "120".into()
    }));
}

#[test]
fn given_daily_heartbeat_when_interval_set_to_60_s_then_next_heartbeat_rescheduled() {
    let mut csms = Csms::booted();

    csms.configure(1_000, "HeartbeatInterval", "60");

    assert_eq!(csms.charge_point.next_deadline(), Some(MINUTE));
}

#[test]
fn given_60_s_heartbeat_when_it_is_due_then_sent_and_next_one_scheduled() {
    let mut csms = Csms::booted();
    csms.configure(1_000, "HeartbeatInterval", "60");

    csms.input(MINUTE, Input::Tick);

    assert_eq!(csms.sent("Heartbeat").len(), 1);
    assert_eq!(csms.charge_point.next_deadline(), Some(2 * MINUTE));
}

#[test]
fn given_booted_charge_point_when_nothing_configured_then_daily_heartbeat() {
    let csms = Csms::booted();

    let deadline = csms.charge_point.next_deadline();

    assert_eq!(deadline, Some(86_400_000));
}

#[test]
fn given_30_s_interval_and_energy_only_when_the_interval_passes_then_one_energy_sample() {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.configure(0, "MeterValueSampleInterval", "30");
    csms.configure(0, "MeterValuesSampledData", "Energy.Active.Import.Register");

    csms.input(30_000, Input::Tick);

    let sampled = &csms.sent("MeterValues")[0]["meterValue"][0]["sampledValue"];
    assert_eq!(sampled.as_array().unwrap().len(), 1);
    assert_eq!(sampled[0]["value"], "92");
}

#[test]
fn given_charging_ev_when_sample_interval_set_to_30_s_then_next_sample_due_in_30_s() {
    let mut csms = Csms::charging(0, EvSpec::default());

    csms.configure(0, "MeterValueSampleInterval", "30");

    assert_eq!(csms.charge_point.next_deadline(), Some(30_000));
}

#[test]
fn given_zero_sample_interval_when_charging_continues_then_no_meter_values() {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.configure(0, "MeterValueSampleInterval", "0");
    let finish = csms.charge_point.next_deadline().unwrap();

    csms.input(finish, Input::Tick);

    assert!(csms.sent("MeterValues").is_empty());
}

#[test]
fn given_booted_charge_point_when_2_0_1_call_received_then_not_implemented() {
    let mut csms = Csms::booted();

    let reply = csms.call(0, "GetVariables", json!({}));

    match reply {
        Frame::CallError { id, code, .. } => {
            assert_eq!(id, "csms-1");
            assert_eq!(code, "NotImplemented");
        }
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn given_booted_charge_point_when_invalid_csms_calls_received_then_the_ocpp_j_4_2_3_error_codes() {
    let cases = [
        ("Reset", json!([]), "FormationViolation"),
        (
            "Reset",
            json!({"type": "Soft", "now": true}),
            "FormationViolation",
        ),
        ("Reset", json!({}), "OccurenceConstraintViolation"),
        (
            "GetConfiguration",
            json!({"key": 5}),
            "TypeConstraintViolation",
        ),
        (
            "Reset",
            json!({"type": "Gentle"}),
            "PropertyConstraintViolation",
        ),
        (
            "UnlockConnector",
            json!({"connectorId": -1}),
            "PropertyConstraintViolation",
        ),
        (
            "ChangeConfiguration",
            json!({"key": "k".repeat(51), "value": "1"}),
            "PropertyConstraintViolation",
        ),
    ];
    for (action, payload, expected) in cases {
        let mut csms = Csms::booted();

        let reply = csms.call(0, action, payload.clone());

        match reply {
            Frame::CallError { code, .. } => assert_eq!(code, expected, "{action} {payload}"),
            other => panic!("{action} {payload}: expected an error, got {other:?}"),
        }
        assert!(
            matches!(
                csms.events[..],
                [EventKind::Link(LinkEvent::InvalidPayload { .. })]
            ),
            "{action} {payload}"
        );
    }
}

#[test]
fn given_booted_charge_point_when_settings_listed_then_they_match_get_configuration() {
    let mut csms = Csms::booted();
    let payload = result(csms.call(0, "GetConfiguration", json!({})));

    let settings = csms.charge_point.settings();

    let listed: Vec<Value> = settings
        .iter()
        .map(|setting| match &setting.value {
            Some(value) => {
                json!({"key": setting.key, "readonly": setting.readonly, "value": value})
            }
            None => json!({"key": setting.key, "readonly": setting.readonly}),
        })
        .collect();
    assert_eq!(Value::Array(listed), payload["configurationKey"]);
}

fn configure(key: &str, value: &str) -> Intent {
    Intent::Configure {
        key: key.into(),
        value: value.into(),
    }
}

#[test]
fn given_booted_charge_point_when_configure_intent_then_the_value_changes_and_configuration_changed_is_emitted()
 {
    let mut csms = Csms::booted();

    csms.intent(0, configure("MeterValueSampleInterval", "15"));

    assert_eq!(value_of(&mut csms, "MeterValueSampleInterval"), "15");
    assert!(csms.events.contains(&EventKind::ConfigurationChanged {
        key: "MeterValueSampleInterval".into(),
        value: "15".into(),
    }));
}

#[test]
fn given_booted_charge_point_when_configure_intent_on_a_read_only_key_then_rejected_as_read_only_while_connected()
 {
    let mut csms = Csms::booted();

    csms.intent(0, configure("NumberOfConnectors", "4"));

    assert!(
        csms.events
            .contains(&EventKind::Link(LinkEvent::IntentRejected {
                connector: None,
                reason: "NumberOfConnectors refused: read-only while connected".into(),
            }))
    );
}

#[test]
fn given_booted_charge_point_when_configure_intent_on_an_unknown_key_then_intent_rejected() {
    let mut csms = Csms::booted();

    csms.intent(0, configure("NoSuchKey", "1"));

    assert!(
        csms.events
            .contains(&EventKind::Link(LinkEvent::IntentRejected {
                connector: None,
                reason: "NoSuchKey refused (NotSupported)".into(),
            }))
    );
}

#[test]
fn given_booted_charge_point_when_configure_intent_sets_the_authorization_key_then_its_value_is_not_emitted()
 {
    let mut csms = Csms::booted();

    csms.intent(
        0,
        configure("AuthorizationKey", "00112233445566778899aabbccddeeff"),
    );

    assert!(
        !csms
            .events
            .iter()
            .any(|event| matches!(event, EventKind::ConfigurationChanged { .. }))
    );
}

#[test]
fn given_disconnected_charge_point_when_configure_intent_on_a_read_only_limit_then_written_and_kept_after_reconnect()
 {
    let mut csms = Csms::booted();
    csms.disconnect(0);

    csms.intent(0, configure("LocalAuthListMaxLength", "5"));

    assert!(csms.events.contains(&EventKind::ConfigurationChanged {
        key: "LocalAuthListMaxLength".into(),
        value: "5".into(),
    }));
    csms.reconnect(0);
    assert_eq!(value_of(&mut csms, "LocalAuthListMaxLength"), "5");
}

#[test]
fn given_disconnected_charge_point_when_configure_intent_on_number_of_connectors_then_rejected_as_derived()
 {
    let mut csms = Csms::booted();
    csms.disconnect(0);

    csms.intent(0, configure("NumberOfConnectors", "4"));

    assert!(
        csms.events
            .contains(&EventKind::Link(LinkEvent::IntentRejected {
                connector: None,
                reason: "NumberOfConnectors refused: derived from the station".into(),
            }))
    );
}

#[test]
fn given_disconnected_charge_point_when_a_read_only_limit_is_set_to_text_then_rejected_as_invalid()
{
    let mut csms = Csms::booted();
    csms.disconnect(0);

    csms.intent(0, configure("LocalAuthListMaxLength", "many"));

    assert!(
        csms.events
            .contains(&EventKind::Link(LinkEvent::IntentRejected {
                connector: None,
                reason: "LocalAuthListMaxLength refused: not a valid value".into(),
            }))
    );
}
