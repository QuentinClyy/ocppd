use crate::api::event::EventKind;
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::{KEY, result, security_events};
use crate::fixtures::x509::authority::Authority;
use crate::v16::messages::core::configuration::ConfigurationStatus;
use serde_json::json;

fn keyed() -> Csms {
    let mut csms = Csms::booted();
    csms.configure(0, "AuthorizationKey", KEY);
    csms
}

fn profile_1() -> Csms {
    let mut csms = keyed();
    csms.configure(0, "SecurityProfile", "1");
    csms
}

fn profile_1_with_root() -> Csms {
    let mut csms = profile_1();
    let root = json!({
        "certificateType": "CentralSystemRootCertificate",
        "certificate": Authority::new("CSMS root").pem()
    });
    result(csms.call(0, "InstallCertificate", root));
    csms
}

fn keyed_at_profile(profile: &str) -> Csms {
    let mut csms = Csms::booted();
    csms.charge_point.configure("AuthorizationKey", KEY);
    csms.charge_point.configure("SecurityProfile", profile);
    csms
}

#[test]
fn given_no_authorization_key_when_profile_1_set_then_rejected_per_a05_fr_02() {
    let mut csms = Csms::booted();

    let status = csms.configure(0, "SecurityProfile", "1");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_authorization_key_when_profile_1_set_then_accepted_per_a05_fr_02() {
    let mut csms = keyed();

    let status = csms.configure(0, "SecurityProfile", "1");

    assert_eq!(status, "Accepted");
}

#[test]
fn given_profile_1_when_profile_1_or_0_set_then_rejected_per_a05_fr_01() {
    let mut csms = profile_1();

    let statuses = [
        csms.configure(0, "SecurityProfile", "1"),
        csms.configure(0, "SecurityProfile", "0"),
    ];

    assert_eq!(statuses, [json!("Rejected"), json!("Rejected")]);
}

#[test]
fn given_profile_1_without_csms_root_when_profile_2_set_then_rejected_per_a05_fr_03() {
    let mut csms = profile_1();

    let status = csms.configure(0, "SecurityProfile", "2");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_profile_1_with_csms_root_when_profile_2_set_then_accepted_per_a05_fr_03() {
    let mut csms = profile_1_with_root();

    let status = csms.configure(0, "SecurityProfile", "2");

    assert_eq!(status, "Accepted");
}

#[test]
fn given_profile_2_without_client_certificate_when_profile_3_set_then_rejected_per_a05_fr_04() {
    let mut csms = profile_1_with_root();
    csms.configure(0, "SecurityProfile", "2");

    let statuses = [
        csms.configure(0, "SecurityProfile", "3"),
        csms.configure(0, "SecurityProfile", "4"),
    ];

    assert_eq!(statuses, [json!("Rejected"), json!("Rejected")]);
}

#[test]
fn given_booted_charge_point_when_another_key_changed_then_connection_kept() {
    let mut csms = Csms::booted();

    csms.configure(0, "HeartbeatInterval", "300");

    assert!(!csms.closed, "other keys keep the connection");
}

#[test]
fn given_booted_charge_point_when_authorization_key_changed_then_reported_and_reconnects_per_a01() {
    let mut csms = Csms::booted();

    csms.configure(0, "AuthorizationKey", KEY);

    assert_eq!(
        security_events(&csms),
        vec![json!("ReconfigurationOfSecurityParameters")],
        "sent before closing"
    );
    assert!(csms.closed);
}

#[test]
fn given_changed_authorization_key_when_reconnected_then_no_boot_per_a05_fr_06() {
    let mut csms = keyed();
    csms.disconnect(1_000);

    csms.reconnect(2_000);

    assert!(csms.sent("BootNotification").is_empty());
}

#[test]
fn given_booted_charge_point_when_invalid_authorization_keys_set_then_rejected() {
    let mut csms = Csms::booted();

    let statuses = [
        csms.configure(0, "AuthorizationKey", "short"),
        csms.configure(0, "AuthorizationKey", "0123456789abcdef"),
        csms.configure(0, "AuthorizationKey", &"zz".repeat(16)),
    ];

    assert_eq!(
        statuses,
        [json!("Rejected"), json!("Rejected"), json!("Rejected")]
    );
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

#[test]
fn given_no_authorization_key_when_connecting_then_no_basic_auth() {
    let csms = Csms::booted();

    let password = csms.charge_point.basic_auth_password();

    assert_eq!(password, None);
}

#[test]
fn given_authorization_key_at_profile_0_when_connecting_then_no_basic_auth() {
    let csms = keyed_at_profile("0");

    let password = csms.charge_point.basic_auth_password();

    assert_eq!(password, None);
}

#[test]
fn given_authorization_key_at_profiles_1_and_2_when_connecting_then_basic_auth_with_the_decoded_key()
 {
    for profile in ["1", "2"] {
        let csms = keyed_at_profile(profile);

        let password = csms.charge_point.basic_auth_password();

        assert_eq!(password, Some(b"0123456789abcdef".to_vec()), "{profile}");
    }
}

#[test]
fn given_authorization_key_at_profile_3_when_connecting_then_no_basic_auth() {
    let csms = keyed_at_profile("3");

    let password = csms.charge_point.basic_auth_password();

    assert_eq!(password, None, "profile 3 uses certificates");
}

#[test]
fn given_each_profile_when_connecting_then_tls_only_from_profile_2() {
    for (profile, tls) in [("0", false), ("1", false), ("2", true), ("3", true)] {
        let mut csms = Csms::booted();
        csms.charge_point.configure("SecurityProfile", profile);

        let required = csms.charge_point.requires_tls();

        assert_eq!(required, tls, "profile {profile}");
    }
}

#[test]
fn given_each_profile_when_connecting_then_client_certificate_only_at_profile_3() {
    for (profile, needed) in [("2", false), ("3", true)] {
        let mut csms = Csms::booted();
        csms.charge_point.configure("SecurityProfile", profile);

        let required = csms.charge_point.requires_client_certificate();

        assert_eq!(required, needed, "profile {profile}");
    }
}

#[test]
fn given_new_charge_point_when_configured_at_spawn_then_value_rules_apply_but_not_a05_prerequisites()
 {
    let mut csms = Csms::booted();

    let statuses = [
        csms.charge_point.configure("SecurityProfile", "2"),
        csms.charge_point.configure("SecurityProfile", "1"),
        csms.charge_point.configure("Nope", "1"),
    ];

    assert_eq!(
        statuses,
        [
            ConfigurationStatus::Accepted,
            ConfigurationStatus::Rejected,
            ConfigurationStatus::NotSupported
        ]
    );
}
