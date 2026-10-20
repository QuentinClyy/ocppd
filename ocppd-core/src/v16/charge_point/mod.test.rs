use crate::api::event::EventKind;
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::{KEY, result, security_events};
use serde_json::json;

fn keyed() -> Csms {
    let mut csms = Csms::booted();
    csms.configure(0, "AuthorizationKey", KEY);
    csms
}

#[test]
fn given_authorization_key_when_profile_1_set_then_accepted_per_a05_fr_02() {
    let mut csms = keyed();

    let status = csms.configure(0, "SecurityProfile", "1");

    assert_eq!(status, "Accepted");
}

#[test]
fn given_booted_charge_point_when_another_key_changed_then_connection_kept() {
    let mut csms = Csms::booted();

    csms.configure(0, "HeartbeatInterval", "300");

    assert!(!csms.closed, "other keys keep the connection");
}

#[test]
fn given_changed_authorization_key_when_reconnected_then_no_boot_per_a05_fr_06() {
    let mut csms = keyed();
    csms.disconnect(1_000);

    csms.reconnect(2_000);

    assert!(csms.sent("BootNotification").is_empty());
}

#[test]
fn given_booted_charge_point_when_hex_authorization_key_set_then_accepted_and_never_echoed() {
    let mut csms = Csms::booted();

    let status = csms.configure(0, "AuthorizationKey", KEY);

    assert_eq!(status, "Accepted");
    let echoed = csms.events.iter();
    assert!(
        !echoed
            .into_iter()
            .any(|kind| matches!(kind, EventKind::ConfigurationChanged { .. }))
    );
    assert_eq!(
        security_events(&csms),
        vec![json!("ReconfigurationOfSecurityParameters")]
    );
}

#[test]
fn given_authorization_key_when_read_then_listed_without_its_value() {
    let mut csms = keyed();
    let request = json!({"key": ["AuthorizationKey"]});

    let payload = result(csms.call(0, "GetConfiguration", request));

    assert_eq!(
        payload["configurationKey"],
        json!([{"key": "AuthorizationKey", "readonly": false}])
    );
}
