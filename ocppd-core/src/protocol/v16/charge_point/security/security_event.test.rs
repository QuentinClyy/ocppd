use crate::api::io::{ConnectFailure, Input};
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::{KEY, result, security_events};
use serde_json::{Value, json};

fn reset(kind: &str) -> Csms {
    let mut csms = Csms::booted();
    result(csms.call(0, "Reset", json!({"type": kind})));
    csms
}

fn booted_again(csms: &mut Csms) {
    csms.reconnect(1_000);
    csms.answer(
        1_000,
        json!({"status": "Accepted", "currentTime": "2026-10-20T00:00:00Z", "interval": 300}),
    );
}

fn refused_certificate_events(csms: &Csms) -> Vec<Value> {
    let events = csms.sent("SecurityEventNotification").into_iter();
    events
        .filter(|event| event["type"] == "InvalidCentralSystemCertificate")
        .collect()
}

fn outage(csms: &mut Csms, start: u64, failure: ConnectFailure) {
    csms.disconnect(start);
    csms.input(start + 1_000, Input::ConnectFailed(failure, String::new()));
    csms.reconnect(start + 10_000);
}

fn refused_twice_offline() -> Csms {
    let mut csms = Csms::booted();
    csms.disconnect(1_000);
    let refused = Input::ConnectFailed(ConnectFailure::CsmsCertificate, String::new());
    csms.input(2_000, refused.clone());
    csms.input(9_000, refused);
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
fn given_soft_or_hard_reset_when_booted_again_then_reset_or_reboot_then_startup_reported_per_a04_fr_02()
 {
    for kind in ["Soft", "Hard"] {
        let mut csms = reset(kind);

        booted_again(&mut csms);

        assert_eq!(
            security_events(&csms),
            vec![json!("ResetOrReboot"), json!("StartupOfTheDevice")],
            "{kind}"
        );
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

#[test]
fn given_refused_csms_certificates_offline_when_still_offline_then_nothing_reported() {
    let csms = refused_twice_offline();

    let events = refused_certificate_events(&csms);

    assert!(events.is_empty(), "offline");
}

#[test]
fn given_refused_csms_certificates_offline_when_back_online_then_reported_once_per_tc_078() {
    let mut csms = refused_twice_offline();

    csms.reconnect(20_000);

    let events = refused_certificate_events(&csms);
    assert_eq!(events.len(), 1, "once per outage");
    assert_eq!(events[0]["timestamp"], "1970-01-01T00:00:02Z");
}

#[test]
fn given_booted_charge_point_when_outage_with_other_failures_then_no_certificate_event() {
    let mut csms = Csms::booted();

    outage(&mut csms, 1_000, ConnectFailure::Other);

    assert!(refused_certificate_events(&csms).is_empty());
}

#[test]
fn given_reported_refused_certificate_when_another_outage_refuses_it_then_reported_again() {
    let mut csms = Csms::booted();
    outage(&mut csms, 1_000, ConnectFailure::CsmsCertificate);

    outage(&mut csms, 30_000, ConnectFailure::CsmsCertificate);

    assert_eq!(refused_certificate_events(&csms).len(), 2);
}
