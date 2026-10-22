use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::charging_profiles::{AMPS_TO_W, entries, offered_w, profile};
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn start(run: &mut Run, profile: Value) -> Value {
    let request = json!({
        "idToken": {"idToken": "APP-USER-1", "type": "Central"},
        "remoteStartId": 7,
        "evseId": 1,
        "chargingProfile": profile,
    });
    let response = run.result(0, "RequestStartTransaction", request);
    run.answer_all(0);
    response
}

fn plug_in(run: &mut Run) {
    let ev = EvSpec::default();
    run.input(0, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(0);
}

fn reported_transaction(run: &mut Run) -> Value {
    let request = json!({"requestId": 1, "chargingProfile": {"chargingProfileId": [5]}});
    run.result(0, "GetChargingProfiles", request);
    let (_, _, report) = run.last_call();
    report["chargingProfile"][0]["transactionId"].clone()
}

fn started_transaction(run: &Run) -> Value {
    let started = run.calls.iter().find(|(_, action, payload)| {
        action == "TransactionEvent" && payload["eventType"] == "Started"
    });
    started.unwrap().2["transactionInfo"]["transactionId"].clone()
}

fn power_path_closed() -> Run {
    let mut run = Run::accepted(86_400);
    assert!(
        run.station
            .configure("TxCtrlr.TxStartPoint", "PowerPathClosed")
    );
    run
}

#[test]
fn given_power_path_closed_start_point_when_remote_start_with_tx_profile_then_accepted_per_f01_fr_10_and_tc_k_37()
 {
    let mut run = power_path_closed();

    let response = start(&mut run, profile(5, "TxProfile", &[(0, 8.0)]));

    assert_eq!(response["status"], "Accepted");
}

#[test]
fn given_remote_start_with_tx_profile_when_ev_plugged_then_the_profile_limits_the_started_transaction_per_f01_fr_10_and_tc_k_37()
 {
    let mut run = power_path_closed();
    start(&mut run, profile(5, "TxProfile", &[(0, 8.0)]));

    plug_in(&mut run);

    assert_eq!(offered_w(&run, 0), Some(8.0 * AMPS_TO_W));
    assert_eq!(entries(&mut run), "1");
    let transaction_id = started_transaction(&run);
    assert_eq!(reported_transaction(&mut run), transaction_id);
}

#[test]
fn given_plugged_ev_when_remote_start_with_tx_profile_then_the_profile_applies_at_once_per_f02_fr_18()
 {
    let mut run = Run::accepted(86_400);
    plug_in(&mut run);

    start(&mut run, profile(5, "TxProfile", &[(0, 8.0)]));

    assert_eq!(offered_w(&run, 0), Some(8.0 * AMPS_TO_W));
    let transaction_id = started_transaction(&run);
    assert_eq!(reported_transaction(&mut run), transaction_id);
}

#[test]
fn given_booted_station_when_remote_start_with_invalid_profiles_then_each_rejected_without_a_transaction_per_f01_fr_09_fr_11_and_fr_26()
 {
    let mut run = Run::accepted(86_400);
    let default = profile(5, "TxDefaultProfile", &[(0, 8.0)]);
    let mut with_transaction = profile(5, "TxProfile", &[(0, 8.0)]);
    with_transaction["transactionId"] = json!("tx-1");
    let late_start = profile(5, "TxProfile", &[(10, 8.0)]);
    let cases = [
        ("F01.FR.09", default, "InvalidProfile"),
        ("F01.FR.11", with_transaction, "InvalidProfile"),
        ("F01.FR.26", late_start, "InvalidSchedule"),
    ];

    let responses: Vec<_> = cases
        .iter()
        .map(|(_, profile, _)| start(&mut run, profile.clone()))
        .collect();

    for ((requirement, _, reason), response) in cases.iter().zip(&responses) {
        assert_eq!(response["status"], "Rejected", "{requirement}");
        assert_eq!(
            response["statusInfo"]["reasonCode"], *reason,
            "{requirement}"
        );
    }
    assert!(run.calls.is_empty(), "no transaction started");
}
