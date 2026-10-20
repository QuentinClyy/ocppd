use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::{KEY, result};
use serde_json::json;

fn reset(kind: &str) -> Csms {
    let mut csms = Csms::booted();
    result(csms.call(0, "Reset", json!({"type": kind})));
    csms
}

#[test]
fn given_booted_charge_point_when_reset_then_closed() {
    for kind in ["Soft", "Hard"] {
        let csms = reset(kind);

        let closed = csms.closed;

        assert!(closed, "{kind}");
    }
}

fn keyed() -> Csms {
    let mut csms = Csms::booted();
    csms.configure(0, "AuthorizationKey", KEY);
    csms
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
