use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::reliability::{configure, events, last, plug_in, swipe};
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::{Value, json};

fn configure_all(run: &mut Run, settings: &[(&str, &str)]) {
    for (key, value) in settings {
        assert!(run.station.configure(key, value), "{key}");
    }
}

fn plug_in_with(run: &mut Run, now: u64, ev: EvSpec) {
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
}

fn swipe_card(run: &mut Run, now: u64) {
    let intent = Intent::Authorize {
        connector: None,
        id_tag: "TAG1".into(),
    };
    run.input(now, Input::Intent(intent));
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(now);
}

fn last_event(run: &Run) -> (Value, Value, Value, Value) {
    let (_, _, event) = run
        .calls
        .iter()
        .rfind(|(_, action, _)| action == "TransactionEvent")
        .expect("no TransactionEvent");
    let info = &event["transactionInfo"];
    (
        event["eventType"].clone(),
        event["triggerReason"].clone(),
        info["chargingState"].clone(),
        info["stoppedReason"].clone(),
    )
}

fn unplugged_while_authorized(run: &mut Run, unplugged: u64) {
    configure_all(
        run,
        &[
            ("TxCtrlr.TxStopPoint", "Authorized"),
            ("TxCtrlr.StopTxOnEVSideDisconnect", "false"),
        ],
    );
    plug_in_with(run, 0, EvSpec::default());
    swipe_card(run, 0);
    run.input(unplugged, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(unplugged);
}

fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

#[test]
fn given_energy_transfer_start_point_and_authorized_id_token_when_ev_plugged_then_started_while_charging_per_e01_fr_06_and_tc_e_02()
 {
    let mut run = Run::accepted(86_400);
    configure_all(&mut run, &[("TxCtrlr.TxStartPoint", "EnergyTransfer")]);
    swipe_card(&mut run, 0);

    plug_in_with(&mut run, 1_000, EvSpec::default());

    assert_eq!(
        last_event(&run),
        (
            json!("Started"),
            json!("ChargingStateChanged"),
            json!("Charging"),
            json!(null)
        )
    );
}

#[test]
fn given_energy_transfer_stop_point_when_ev_battery_full_then_ended_as_stopped_by_ev_per_e06_fr_07_and_tc_e_22()
 {
    let mut run = Run::accepted(86_400);
    configure_all(
        &mut run,
        &[
            ("TxCtrlr.TxStopPoint", "EnergyTransfer"),
            ("SampledDataCtrlr.TxUpdatedInterval", "0"),
        ],
    );
    let ev = EvSpec {
        soc: 0.99,
        ..EvSpec::default()
    };
    plug_in_with(&mut run, 0, ev);
    swipe_card(&mut run, 0);
    let full = run.station.next_deadline().unwrap();

    tick(&mut run, full);

    assert_eq!(
        last_event(&run),
        (
            json!("Ended"),
            json!("ChargingStateChanged"),
            json!("EVConnected"),
            json!("StoppedByEV")
        )
    );
}

#[test]
fn given_ev_side_unplug_when_just_before_the_connection_timeout_then_transaction_kept_per_e10_fr_02()
 {
    let mut run = Run::accepted(86_400);
    let unplugged = 10 * MINUTE;
    unplugged_while_authorized(&mut run, unplugged);

    tick(&mut run, unplugged + MINUTE - 1);

    assert_eq!(last_event(&run).0, json!("Updated"), "not yet timed out");
}

#[test]
fn given_ev_side_unplug_when_the_connection_timeout_expires_then_ended_by_timeout_per_e10_fr_02_and_tc_e_27()
 {
    let mut run = Run::accepted(86_400);
    let unplugged = 10 * MINUTE;
    unplugged_while_authorized(&mut run, unplugged);

    tick(&mut run, unplugged + MINUTE);

    assert_eq!(
        last_event(&run),
        (
            json!("Ended"),
            json!("EVConnectTimeout"),
            json!(null),
            json!("Timeout")
        )
    );
}

#[test]
fn given_authorized_id_token_when_full_ev_plugged_then_plug_in_reported_before_suspended_ev_per_tc_e_38()
 {
    let mut run = Run::accepted(86_400);
    swipe_card(&mut run, 0);
    let full = EvSpec {
        soc: 1.0,
        ..EvSpec::default()
    };

    plug_in_with(&mut run, 1_000, full);

    let states: Vec<_> = run
        .calls
        .iter()
        .filter(|(_, action, _)| action == "TransactionEvent")
        .map(|(_, _, event)| {
            (
                event["triggerReason"].clone(),
                event["transactionInfo"]["chargingState"].clone(),
            )
        })
        .collect();
    assert_eq!(
        states[1..],
        [
            (json!("CablePluggedIn"), json!("EVConnected")),
            (json!("ChargingStateChanged"), json!("SuspendedEV")),
        ]
    );
}

fn charging(run: &mut Run, now: u64) {
    plug_in(run, now);
    run.answer_all(now);
    swipe(run, now, "04A2B3C4");
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "FLEET", "type": "Central"}}}));
    run.answer_all(now);
}

fn l(
    event_type: &str,
    trigger: &str,
    state: Value,
    reason: Value,
) -> (String, String, Value, Value) {
    (event_type.into(), trigger.into(), state, reason)
}

fn authorized_with_event_pending(run: &mut Run) {
    plug_in(run, 0);
    run.answer_all(0);
    swipe(run, 0, "04A2B3C4");
    run.answer(0, json!({"idTokenInfo": {"status": "Accepted"}}));
}

fn blocked(run: &mut Run) {
    run.answer(0, json!({"idTokenInfo": {"status": "Blocked"}}));
    run.answer_all(0);
}

fn master_pass(run: &mut Run) {
    configure(run, "AuthCtrlr.MasterPassGroupId", "POLICE");
}

fn police(run: &mut Run, now: u64) {
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "POLICE", "type": "Central"}}}));
    run.answer_all(now);
}

#[test]
fn given_authorized_transaction_when_its_id_token_comes_back_blocked_then_ended_as_deauthorized_per_e05_fr_10()
 {
    let mut run = Run::accepted(86_400);
    authorized_with_event_pending(&mut run);

    blocked(&mut run);

    assert_eq!(
        last(&run),
        l(
            "Ended",
            "Deauthorized",
            json!("EVConnected"),
            json!("DeAuthorized")
        )
    );
}

#[test]
fn given_ev_connected_stop_point_when_id_token_comes_back_blocked_then_only_deauthorized_per_e05_fr_09()
 {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "TxCtrlr.TxStopPoint", "EVConnected");
    authorized_with_event_pending(&mut run);

    blocked(&mut run);

    assert_eq!(
        last(&run),
        l("Updated", "Deauthorized", json!("EVConnected"), json!(null))
    );
}

#[test]
fn given_no_stop_on_invalid_id_when_id_token_comes_back_blocked_then_energy_suspended_per_e05_fr_02()
 {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "TxCtrlr.StopTxOnInvalidId", "false");
    authorized_with_event_pending(&mut run);

    blocked(&mut run);

    assert_eq!(
        last(&run),
        l(
            "Updated",
            "ChargingStateChanged",
            json!("SuspendedEVSE"),
            json!(null)
        )
    );
}

#[test]
fn given_charging_ev_when_master_pass_presented_then_authorize_sent_per_c16_fr_02() {
    let mut run = Run::accepted(86_400);
    master_pass(&mut run);
    charging(&mut run, 0);

    swipe(&mut run, MINUTE, "OFFICER1");

    assert_eq!(run.last_call().1, "Authorize");
}

#[test]
fn given_charging_ev_and_presented_master_pass_when_accepted_then_every_transaction_stopped_per_c16_fr_02_and_fr_08()
 {
    let mut run = Run::accepted(86_400);
    master_pass(&mut run);
    charging(&mut run, 0);
    swipe(&mut run, MINUTE, "OFFICER1");

    police(&mut run, MINUTE);

    assert_eq!(
        last(&run),
        l(
            "Ended",
            "StopAuthorized",
            json!("EVConnected"),
            json!("MasterPass")
        )
    );
}

#[test]
fn given_authorized_start_point_and_presented_master_pass_when_accepted_then_no_transaction_per_c16_fr_03()
 {
    let mut run = Run::accepted(86_400);
    master_pass(&mut run);
    configure(&mut run, "TxCtrlr.TxStartPoint", "Authorized");
    swipe(&mut run, 0, "OFFICER1");

    police(&mut run, 0);

    assert!(events(&run).is_empty());
}

#[test]
fn given_charging_ev_when_id_token_of_the_same_group_accepted_then_stopped_with_its_id_token_per_e07_fr_02_and_tc_c_39()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);
    swipe(&mut run, MINUTE, "04B5C6D7");

    run.answer(MINUTE, json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "FLEET", "type": "Central"}}}));
    run.answer_all(MINUTE);

    assert_eq!(
        events(&run).pop().unwrap()["idToken"],
        json!({"idToken": "04B5C6D7", "type": "ISO14443"})
    );
    assert_eq!(
        last(&run),
        l(
            "Ended",
            "StopAuthorized",
            json!("EVConnected"),
            json!("Local")
        )
    );
}

#[test]
fn given_charging_ev_when_reset_on_idle_then_scheduled_per_b12_fr_01() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);

    let status = run.result(0, "Reset", json!({"type": "OnIdle"}))["status"].clone();

    assert_eq!(status, "Scheduled");
    assert!(!run.closed);
}

#[test]
fn given_scheduled_reset_when_transaction_ends_then_reboots_per_b12_fr_01() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);
    run.result(0, "Reset", json!({"type": "OnIdle"}));

    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(MINUTE);

    assert!(run.closed, "reboots once the transaction ended");
}

#[test]
fn given_charging_ev_when_reset_immediate_then_transaction_ended_first_and_reboots_per_b12_fr_02_and_fr_04()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);

    let status = run.result(0, "Reset", json!({"type": "Immediate"}))["status"].clone();
    run.answer_all(0);

    assert_eq!(status, "Accepted");
    assert_eq!(
        last(&run),
        l(
            "Ended",
            "ResetCommand",
            json!("EVConnected"),
            json!("ImmediateReset")
        )
    );
    assert!(run.closed);
}
