use crate::api::io::{ConnectFailure, Input};
use crate::fixtures::v201::station::{Run, boot};
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

const PASSWORD: &str = "correct-horse-battery";

fn pushed(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "SecurityEventNotification")
        .map(|(_, _, payload)| payload["type"].clone())
        .collect()
}

fn logged(run: &Run) -> Vec<String> {
    run.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::SecurityEvent { event_type } => Some(event_type.clone()),
            _ => None,
        })
        .collect()
}

fn set_password(run: &mut Run, password: &str) -> Value {
    let data = json!({"component": {"name": "SecurityCtrlr"}, "variable": {"name": "BasicAuthPassword"}, "attributeValue": password});
    run.result(0, "SetVariables", json!({"setVariableData": [data]}))["setVariableResult"][0]["attributeStatus"].clone()
}

fn booted_new() -> Run {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);
    run.answer(0, boot("Accepted", 86_400));
    run.answer_all(0);
    run
}

fn refused_certificate_events(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .map(|(_, _, payload)| payload.clone())
        .filter(|payload| payload["type"] == "InvalidCsmsCertificate")
        .collect()
}

fn outage(run: &mut Run, start: u64, failure: ConnectFailure) {
    run.input(start, Input::Disconnected);
    run.input(start + 1_000, Input::ConnectFailed(failure));
    run.input(start + 10_000, Input::Connected);
    run.answer_all(start + 10_000);
}

fn profile(run: &mut Run, profile: &str) {
    assert!(
        run.station
            .configure("SecurityCtrlr.SecurityProfile", profile)
    );
}

#[test]
fn given_new_station_when_boot_accepted_then_startup_pushed_per_a04_fr_01() {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);

    run.answer(0, boot("Accepted", 86_400));
    run.answer_all(0);

    assert_eq!(pushed(&run), vec![json!("StartupOfTheDevice")]);
}

#[test]
fn given_started_station_when_reconnected_then_no_new_startup_per_a04_fr_01() {
    let mut run = booted_new();

    run.reconnect(1_000);
    run.answer_all(1_000);

    assert_eq!(pushed(&run).len(), 1, "a reconnect is not a startup");
}

#[test]
fn given_immediate_reset_when_booted_again_then_reset_or_reboot_then_startup_pushed() {
    let mut run = Run::accepted(86_400);
    run.calls.clear();
    run.result(0, "Reset", json!({"type": "Immediate"}));

    run.reconnect(1_000);
    run.answer(1_000, boot("Accepted", 86_400));
    run.answer_all(1_000);

    assert_eq!(
        pushed(&run),
        vec![json!("ResetOrReboot"), json!("StartupOfTheDevice")]
    );
}

#[test]
fn given_unconfirmed_startup_event_when_reconnected_then_sent_again_per_a04_fr_02() {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);
    run.answer(0, boot("Accepted", 86_400));

    run.reconnect(1_000);
    run.answer_all(1_000);

    assert_eq!(
        pushed(&run),
        vec![json!("StartupOfTheDevice"), json!("StartupOfTheDevice")],
        "sent again until the CSMS confirms it"
    );
}

#[test]
fn given_booted_station_when_too_short_password_set_then_rejected_per_a01() {
    let mut run = Run::accepted(86_400);

    let status = set_password(&mut run, "too-short");

    assert_eq!(status, "Rejected", "16 to 40 characters");
}

#[test]
fn given_booted_station_when_password_set_then_accepted_and_only_logged_without_the_secret_per_a01_fr_02_fr_11_and_fr_12()
 {
    let mut run = Run::accepted(86_400);

    let status = set_password(&mut run, PASSWORD);

    assert_eq!(status, "Accepted", "A01.FR.02");
    assert_eq!(
        logged(&run),
        vec!["ReconfigurationOfSecurityParameters"],
        "A01.FR.11"
    );
    assert!(pushed(&run).is_empty(), "not critical, so only logged");
    let leaked = run
        .events
        .iter()
        .any(|kind| format!("{kind:?}").contains(PASSWORD));
    assert!(!leaked, "A01.FR.12");
}

#[test]
fn given_password_set_when_read_then_rejected_as_write_only_per_b06_fr_09() {
    let mut run = Run::accepted(86_400);
    set_password(&mut run, PASSWORD);
    let get = json!({"getVariableData": [{"component": {"name": "SecurityCtrlr"}, "variable": {"name": "BasicAuthPassword"}}]});

    let read = run.result(0, "GetVariables", get)["getVariableResult"][0].clone();

    assert_eq!(read["attributeStatus"], "Rejected");
    assert!(read.get("attributeValue").is_none());
}

#[test]
fn given_password_on_profile_0_when_connecting_then_no_basic_auth_per_a00_fr_202() {
    let mut run = Run::new(1);
    run.station.set_basic_auth_password(PASSWORD);

    let password = run.station.basic_auth_password();

    assert_eq!(password, None);
}

#[test]
fn given_password_on_profile_1_when_connecting_then_basic_auth_without_tls_per_a00_fr_203() {
    let mut run = Run::new(1);
    run.station.set_basic_auth_password(PASSWORD);
    profile(&mut run, "1");

    let password = run.station.basic_auth_password();

    assert_eq!(password, Some(PASSWORD.as_bytes().to_vec()));
    assert!(!run.station.requires_tls());
}

#[test]
fn given_password_on_profile_2_when_connecting_then_basic_auth_over_tls_per_a00_fr_204() {
    let mut run = Run::new(1);
    run.station.set_basic_auth_password(PASSWORD);
    profile(&mut run, "2");

    let password = run.station.basic_auth_password();

    assert!(password.is_some());
    assert!(run.station.requires_tls());
}

#[test]
fn given_password_on_profile_3_when_connecting_then_client_certificate_instead_per_a00_fr_205() {
    let mut run = Run::new(1);
    run.station.set_basic_auth_password(PASSWORD);
    profile(&mut run, "3");

    let password = run.station.basic_auth_password();

    assert_eq!(password, None, "profile 3 uses certificates");
    assert!(run.station.requires_client_certificate());
}

#[test]
fn given_new_station_when_out_of_range_or_read_only_variables_configured_then_all_refused() {
    let mut run = Run::new(1);

    let accepted = [
        run.station.configure("SecurityCtrlr.SecurityProfile", "4"),
        run.station
            .configure("OCPPCommCtrlr.ActiveNetworkProfile", "1"),
        run.station
            .configure("DeviceDataCtrlr.ItemsPerMessage[GetVariables]", "10"),
    ];

    assert_eq!(accepted, [false, false, false]);
}

#[test]
fn given_outage_with_refused_csms_certificates_when_back_online_then_reported_once_per_tc_a_05() {
    let mut run = Run::accepted(86_400);
    run.input(1_000, Input::Disconnected);
    let refused = Input::ConnectFailed(ConnectFailure::CsmsCertificate);
    run.input(2_000, refused.clone());
    run.input(9_000, refused);
    run.input(10_000, Input::ConnectFailed(ConnectFailure::Other));

    run.input(20_000, Input::Connected);
    run.answer_all(20_000);

    let events = refused_certificate_events(&run);
    assert_eq!(events.len(), 1, "once per outage");
    assert_eq!(events[0]["timestamp"], "1970-01-01T00:00:02Z");
}

#[test]
fn given_booted_station_when_outage_with_other_failures_then_no_certificate_event() {
    let mut run = Run::accepted(86_400);

    outage(&mut run, 1_000, ConnectFailure::Other);

    assert!(refused_certificate_events(&run).is_empty());
}

#[test]
fn given_reported_refused_certificate_when_another_outage_refuses_it_then_reported_again() {
    let mut run = Run::accepted(86_400);
    outage(&mut run, 1_000, ConnectFailure::CsmsCertificate);

    outage(&mut run, 30_000, ConnectFailure::CsmsCertificate);

    assert_eq!(refused_certificate_events(&run).len(), 2);
}
