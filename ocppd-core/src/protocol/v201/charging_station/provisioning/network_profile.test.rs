use crate::api::io::{ConnectFailure, Input};
use crate::fixtures::v201::station::{Run, boot};
use crate::fixtures::x509::authority::Authority;
use serde_json::{Value, json};

const SPAWN_URL: &str = "ws://csms.example/ocpp";
const NEW_URL: &str = "wss://new.example/ocpp";

fn spawned(security_profile: &str) -> Run {
    let mut run = Run::new(1);
    assert!(
        run.station
            .configure("SecurityCtrlr.SecurityProfile", security_profile)
    );
    assert!(run.station.set_csms_url(SPAWN_URL));
    run.input(0, Input::Connected);
    run.answer(0, boot("Accepted", 86_400));
    run.answer_all(0);
    run.calls.clear();
    run
}

fn profile(url: &str, security_profile: i32) -> Value {
    json!({"ocppVersion": "OCPP20", "ocppTransport": "JSON", "ocppCsmsUrl": url, "messageTimeout": 30, "securityProfile": security_profile, "ocppInterface": "Wired0"})
}

fn set_profile(run: &mut Run, slot: i32, data: Value) -> Value {
    let request = json!({"configurationSlot": slot, "connectionData": data});
    run.result(0, "SetNetworkProfile", request)
}

fn set_priority(run: &mut Run, value: &str) -> Value {
    let data = json!({"component": {"name": "OCPPCommCtrlr"}, "variable": {"name": "NetworkConfigurationPriority"}, "attributeValue": value});
    run.result(0, "SetVariables", json!({"setVariableData": [data]}))["setVariableResult"][0]
        ["attributeStatus"]
        .clone()
}

fn variable(run: &mut Run, component: &str, name: &str) -> Value {
    let data = json!({"component": {"name": component}, "variable": {"name": name}});
    run.result(0, "GetVariables", json!({"getVariableData": [data]}))["getVariableResult"][0]
        ["attributeValue"]
        .clone()
}

fn install_root(run: &mut Run) {
    let request = json!({"certificateType": "CSMSRootCertificate", "certificate": Authority::new("CSO root").pem()});
    assert_eq!(
        run.result(0, "InstallCertificate", request)["status"],
        "Accepted"
    );
}

fn reboot(run: &mut Run, now: u64) {
    run.result(now, "Reset", json!({"type": "Immediate"}));
    run.reconnect(now);
    run.answer(now, boot("Accepted", 86_400));
    run.answer_all(now);
}

fn second_profile(security_profile: &str, priority: &str) -> Run {
    let mut run = spawned(security_profile);
    set_profile(&mut run, 1, profile("ws://new.example/ocpp", 0));
    set_priority(&mut run, priority);
    run
}

fn upgrade_set() -> Run {
    let mut run = spawned("1");
    install_root(&mut run);
    set_profile(&mut run, 1, profile(NEW_URL, 2));
    set_priority(&mut run, "1,0");
    run
}

fn fail(run: &mut Run, times: usize) {
    for _ in 0..times {
        run.input(0, Input::ConnectFailed(ConnectFailure::Other));
    }
}

#[test]
fn given_profile_1_when_valid_profile_set_then_accepted_per_b09_fr_01() {
    let mut run = spawned("1");

    let reply = set_profile(&mut run, 1, profile(NEW_URL, 2));

    assert_eq!(reply, json!({"status": "Accepted"}));
}

#[test]
fn given_profile_1_when_unknown_slot_set_then_rejected_as_invalid_slot_per_b09_fr_05() {
    let mut run = spawned("1");

    let reply = set_profile(&mut run, 2, profile(NEW_URL, 2));

    assert_eq!(reply["status"], "Rejected");
    assert_eq!(reply["statusInfo"]["reasonCode"], "InvalidConfSlot");
}

#[test]
fn given_profile_0_when_invalid_profiles_set_then_each_rejected_as_invalid_per_b09_fr_02() {
    let mut run = spawned("0");
    let mut old_version = profile(NEW_URL, 2);
    old_version["ocppVersion"] = json!("OCPP16");
    let mut soap = profile(NEW_URL, 2);
    soap["ocppTransport"] = json!("SOAP");
    let invalid = [
        old_version,
        soap,
        profile("http://new.example", 0),
        profile("ws://new.example", 2),
        profile(NEW_URL, 4),
    ];

    let replies: Vec<_> = invalid
        .iter()
        .map(|data| set_profile(&mut run, 1, data.clone()))
        .collect();

    for (data, reply) in invalid.iter().zip(&replies) {
        assert_eq!(
            reply["statusInfo"]["reasonCode"], "InvalidNetworkConf",
            "{data}"
        );
    }
}

#[test]
fn given_profile_1_when_lower_profile_set_then_rejected_as_downgrade_per_b09_fr_04() {
    let mut run = spawned("1");

    let reply = set_profile(&mut run, 1, profile("ws://new.example/ocpp", 0));

    assert_eq!(reply["statusInfo"]["reasonCode"], "NoSecurityDowngrade");
}

#[test]
fn given_empty_slot_when_priority_names_it_then_rejected_per_b10_fr_01() {
    let mut run = spawned("0");

    let status = set_priority(&mut run, "1,0");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_set_slot_when_priority_names_it_then_reboot_required_per_b10_fr_02() {
    let mut run = spawned("0");
    set_profile(&mut run, 1, profile("ws://new.example/ocpp", 0));

    let status = set_priority(&mut run, "1,0");

    assert_eq!(status, "RebootRequired");
}

#[test]
fn given_tls_profile_without_csms_root_when_prioritized_then_rejected_per_a05_fr_02() {
    let mut run = spawned("1");
    set_profile(&mut run, 1, profile(NEW_URL, 2));

    let status = set_priority(&mut run, "1,0");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_tls_profile_and_csms_root_when_prioritized_then_reboot_required_per_a05_fr_02() {
    let mut run = spawned("1");
    set_profile(&mut run, 1, profile(NEW_URL, 2));
    install_root(&mut run);

    let status = set_priority(&mut run, "1,0");

    assert_eq!(status, "RebootRequired");
}

#[test]
fn given_profile_3_without_client_certificate_when_prioritized_then_rejected_per_a05_fr_03() {
    let mut run = spawned("2");
    install_root(&mut run);
    set_profile(&mut run, 1, profile(NEW_URL, 3));

    let status = set_priority(&mut run, "1,0");

    assert_eq!(status, "Rejected");
}

#[test]
fn given_new_priority_when_not_rebooted_then_spawn_url_still_used_per_b10_fr_04() {
    let run = second_profile("0", "1,0");

    let url = run.station.csms_url();

    assert_eq!(url.unwrap(), SPAWN_URL);
}

#[test]
fn given_new_priority_when_rebooted_then_first_entry_used_per_b10_fr_04() {
    let mut run = second_profile("0", "1,0");

    reboot(&mut run, 1_000);

    assert_eq!(run.station.csms_url().unwrap(), "ws://new.example/ocpp");
}

#[test]
fn given_two_entries_when_first_fails_twice_then_still_tried_per_b10_fr_03() {
    let mut run = second_profile("0", "0,1");
    run.input(1_000, Input::Disconnected);

    fail(&mut run, 2);

    assert_eq!(run.station.csms_url().unwrap(), SPAWN_URL);
}

#[test]
fn given_two_entries_when_first_fails_its_attempts_then_second_tried_per_b10_fr_03() {
    let mut run = second_profile("0", "0,1");
    run.input(1_000, Input::Disconnected);

    fail(&mut run, 3);

    assert_eq!(run.station.csms_url().unwrap(), "ws://new.example/ocpp");
}

#[test]
fn given_two_entries_when_both_fail_their_attempts_then_first_tried_again_per_b10_fr_08() {
    let mut run = second_profile("0", "0,1");
    run.input(1_000, Input::Disconnected);

    fail(&mut run, 6);

    assert_eq!(run.station.csms_url().unwrap(), SPAWN_URL);
}

#[test]
fn given_two_entries_when_reconnected_on_the_same_profile_then_no_boot_per_b10_fr_10() {
    let mut run = second_profile("0", "0,1");
    run.input(1_000, Input::Disconnected);

    run.input(2_000, Input::Connected);
    run.answer_all(2_000);

    assert!(
        run.actions()
            .iter()
            .all(|action| action != "BootNotification")
    );
}

#[test]
fn given_two_entries_when_connected_on_the_other_profile_then_boots_again_per_b10_fr_10() {
    let mut run = second_profile("0", "0,1");
    run.input(3_000, Input::Disconnected);
    fail(&mut run, 3);

    run.input(4_000, Input::Connected);

    assert_eq!(run.last_call().1, "BootNotification");
}

#[test]
fn given_higher_profile_prioritized_when_not_rebooted_then_tls_not_yet_required_per_a05_fr_06() {
    let run = upgrade_set();

    let tls = run.station.requires_tls();

    assert!(!tls);
}

#[test]
fn given_higher_profile_prioritized_when_rebooted_then_upgraded_and_lower_slots_dropped_per_a05_fr_06()
 {
    let mut run = upgrade_set();

    reboot(&mut run, 1_000);

    assert!(
        run.station.requires_tls(),
        "the new profile is used to connect"
    );
    assert_eq!(variable(&mut run, "SecurityCtrlr", "SecurityProfile"), "2");
    assert_eq!(
        variable(&mut run, "OCPPCommCtrlr", "ActiveNetworkProfile"),
        "1"
    );
    assert_eq!(
        variable(&mut run, "OCPPCommCtrlr", "NetworkConfigurationPriority"),
        "1"
    );
}

#[test]
fn given_upgraded_profile_when_connections_keep_failing_then_never_back_to_the_lower_profile_per_a05_fr_06()
 {
    let mut run = upgrade_set();
    reboot(&mut run, 1_000);
    run.input(2_000, Input::Disconnected);

    fail(&mut run, 6);

    assert_eq!(run.station.csms_url().unwrap(), NEW_URL);
}
