use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::{KEY, result, security_events};
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

#[test]
fn given_unanswered_non_critical_event_when_reconnected_then_not_resent() {
    let mut csms = Csms::booted();
    csms.answered_by_test.push("SecurityEventNotification");
    csms.configure(0, "AuthorizationKey", KEY);
    csms.expect("SecurityEventNotification");
    csms.disconnect(1_000);

    csms.reconnect(2_000);

    assert_eq!(
        security_events(&csms),
        vec![json!("ReconfigurationOfSecurityParameters")]
    );
}

#[test]
fn given_booted_charge_point_when_plainly_reconnected_then_no_startup_event() {
    let mut csms = Csms::booted();
    csms.disconnect(1_000);

    csms.reconnect(2_000);

    assert!(security_events(&csms).is_empty());
}
