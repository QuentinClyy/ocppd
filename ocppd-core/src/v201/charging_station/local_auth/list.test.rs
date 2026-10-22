use crate::fixtures::v201::local_auth::*;
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::{Value, json};

#[test]
fn given_no_list_sent_when_version_asked_then_0_per_d02_fr_02() {
    let mut run = Run::accepted(86_400);

    let version = version(&mut run);

    assert_eq!(version, 0);
}

#[test]
fn given_booted_station_when_full_list_sent_then_accepted_per_d01_fr_15() {
    let mut run = Run::accepted(86_400);
    let full = list(json!([entry(CARD, "Accepted")]), 3, "Full");

    let status = send(&mut run, full);

    assert_eq!(status, "Accepted");
}

#[test]
fn given_list_version_3_when_differential_removal_sent_then_accepted_per_d01_fr_17_and_tc_d_03() {
    let mut run = list_v3();
    let removal = list(
        json!([{"idToken": {"idToken": "04B5C6D7", "type": "ISO14443"}}]),
        4,
        "Differential",
    );

    let status = send(&mut run, removal);

    assert_eq!(status, "Accepted");
}

#[test]
fn given_list_version_4_when_differential_with_version_4_sent_then_version_mismatch_per_d01_fr_19_and_tc_d_06()
 {
    let mut run = list_v3();
    send(&mut run, list(Value::Null, 4, "Differential"));
    let stale = list(json!([entry("04E5E6E7", "Accepted")]), 4, "Differential");

    let status = send(&mut run, stale);

    assert_eq!(status, "VersionMismatch");
}

#[test]
fn given_booted_station_when_list_with_version_0_sent_then_failed_per_d01_fr_18() {
    let mut run = Run::accepted(86_400);
    let zero = list(json!([entry(CARD, "Accepted")]), 0, "Full");

    let status = send(&mut run, zero);

    assert_eq!(status, "Failed");
}

#[test]
fn given_booted_station_when_list_with_the_same_id_token_twice_sent_then_failed_per_d01_fr_06() {
    let mut run = Run::accepted(86_400);
    let twice = list(
        json!([entry(CARD, "Accepted"), entry(CARD, "Blocked")]),
        5,
        "Full",
    );

    let status = send(&mut run, twice);

    assert_eq!(status, "Failed");
}

#[test]
fn given_list_and_disabled_local_list_when_version_asked_then_0_per_d02_fr_03_and_tc_d_10() {
    let mut run = list_v3();
    configure(&mut run, "LocalAuthListCtrlr.Enabled", "false");

    let version = version(&mut run);

    assert_eq!(version, 0);
}

#[test]
fn given_cached_id_token_listed_as_blocked_when_it_starts_again_then_csms_asked_per_c13_fr_01_and_c14_fr_03()
 {
    let mut run = Run::accepted(86_400);
    session(&mut run, 0, "Accepted");
    send(&mut run, list(json!([entry(CARD, "Blocked")]), 1, "Full"));

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2);
}
