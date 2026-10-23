use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::fixtures::v201::transaction::{
    accept, charging, configure, events, intent, plug_in, s, summaries, summary, swipe,
};
use serde_json::json;

#[test]
fn given_power_path_closed_start_point_when_ev_plugged_then_no_transaction_yet_per_e01_fr_05() {
    let mut run = Run::accepted(300);
    configure(&mut run, "TxCtrlr.TxStartPoint", "PowerPathClosed");

    plug_in(&mut run, 0);

    assert!(events(&run).is_empty());
}

#[test]
fn given_power_path_closed_start_point_and_plugged_ev_when_authorized_then_started_and_charging_per_e01_fr_05()
 {
    let mut run = Run::accepted(300);
    configure(&mut run, "TxCtrlr.TxStartPoint", "PowerPathClosed");
    plug_in(&mut run, 0);
    swipe(&mut run, 1_000, "TAG1");

    accept(&mut run, 1_000);

    assert_eq!(
        summaries(&run),
        vec![
            s("Started", "Authorized", json!("EVConnected")),
            s("Updated", "ChargingStateChanged", json!("Charging")),
        ]
    );
}

#[test]
fn given_ev_unplugged_without_stop_on_ev_side_disconnect_when_plugged_again_then_transaction_resumes_per_e10()
 {
    let mut run = Run::accepted(300);
    configure(&mut run, "TxCtrlr.StopTxOnEVSideDisconnect", "false");
    configure(&mut run, "TxCtrlr.TxStopPoint", "Authorized");
    charging(&mut run, 0);
    intent(&mut run, 1_000, Intent::Unplug { connector: 1 });

    plug_in(&mut run, 2_000);

    assert_eq!(
        summaries(&run)[3..],
        [
            s("Updated", "EVCommunicationLost", json!("Idle")),
            s("Updated", "CablePluggedIn", json!("EVConnected")),
            s("Updated", "ChargingStateChanged", json!("Charging")),
        ]
    );
}

#[test]
fn given_plugged_ev_and_presented_id_token_when_invalid_then_transaction_not_ended_per_e06() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);
    swipe(&mut run, 1_000, "TAG1");

    run.answer(1_000, json!({"idTokenInfo": {"status": "Invalid"}}));
    run.answer_all(1_000);

    assert_eq!(
        summaries(&run),
        vec![s("Started", "CablePluggedIn", json!("EVConnected"))]
    );
}

fn authorizes(run: &Run) -> usize {
    run.actions()
        .iter()
        .filter(|action| *action == "Authorize")
        .count()
}

#[test]
fn given_plugged_ev_when_id_token_presented_then_authorize_sent_per_e02_fr_06() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 1_000);

    swipe(&mut run, 2_000, "TAG1");

    assert_eq!(run.last_call().1, "Authorize");
}

#[test]
fn given_plugged_ev_and_presented_id_token_when_accepted_then_same_transaction_gets_the_id_token_then_charges_per_e02_fr_06_and_fr_20()
 {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 1_000);
    swipe(&mut run, 2_000, "TAG1");

    accept(&mut run, 2_000);

    assert_eq!(
        summaries(&run),
        vec![
            s("Started", "CablePluggedIn", json!("EVConnected")),
            s("Updated", "Authorized", json!(null)),
            s("Updated", "ChargingStateChanged", json!("Charging")),
        ]
    );
    let events = events(&run);
    assert_eq!(events[0]["seqNo"], 0, "E01.FR.07");
    assert_eq!(events[1]["seqNo"], 1);
    assert_eq!(
        events[0]["evse"],
        json!({"id": 1, "connectorId": 1}),
        "E01.FR.16"
    );
    assert!(events[0].get("idToken").is_none());
    assert_eq!(
        events[1]["idToken"],
        json!({"idToken": "TAG1", "type": "Local"})
    );
    assert_eq!(
        events[0]["transactionInfo"]["transactionId"],
        events[1]["transactionInfo"]["transactionId"]
    );
}

#[test]
fn given_authorized_id_token_when_ev_plugged_then_transaction_started_on_authorize_charges_per_e03_fr_01()
 {
    let mut run = Run::accepted(300);
    swipe(&mut run, 1_000, "TAG1");
    accept(&mut run, 1_000);

    plug_in(&mut run, 2_000);

    assert_eq!(
        summaries(&run),
        vec![
            s("Started", "Authorized", json!("Idle")),
            s("Updated", "CablePluggedIn", json!("EVConnected")),
            s("Updated", "ChargingStateChanged", json!("Charging")),
        ]
    );
    assert_eq!(events(&run)[0]["idToken"]["idToken"], "TAG1", "E01.FR.10");
}

#[test]
fn given_charging_ev_when_unplugged_then_transaction_ended_as_ev_disconnected_per_e06_fr_02_and_fr_08()
 {
    let mut run = Run::accepted(300);
    charging(&mut run, 1_000);

    intent(&mut run, 2_000, Intent::Unplug { connector: 1 });

    let ended = events(&run).pop().unwrap();
    assert_eq!(
        summary(&ended),
        s("Ended", "EVCommunicationLost", json!("Idle"))
    );
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "EVDisconnected");
}

#[test]
fn given_charging_ev_when_same_id_token_presented_then_stopped_locally_without_authorize_per_e07_fr_01_and_fr_02()
 {
    let mut run = Run::accepted(300);
    charging(&mut run, 1_000);
    let before = authorizes(&run);

    swipe(&mut run, 2_000, "tag1");
    run.answer_all(2_000);

    assert_eq!(authorizes(&run), before, "no second Authorize");
    let ended = events(&run).pop().unwrap();
    assert_eq!(
        summary(&ended),
        s("Ended", "StopAuthorized", json!("EVConnected"))
    );
    assert_eq!(
        ended["transactionInfo"]["stoppedReason"], "Local",
        "E07.FR.05"
    );
    assert_eq!(ended["idToken"]["idToken"], "tag1", "E07.FR.02");
}

#[test]
fn given_authorization_stopped_before_any_transaction_when_ev_plugged_then_started_without_id_token()
 {
    let mut run = Run::accepted(300);
    configure(&mut run, "TxCtrlr.TxStartPoint", "EVConnected");
    swipe(&mut run, 1_000, "TAG1");
    accept(&mut run, 1_000);
    swipe(&mut run, 2_000, "TAG1");

    plug_in(&mut run, 3_000);

    let started = events(&run).pop().unwrap();
    assert_eq!(started["triggerReason"], "CablePluggedIn");
    assert!(started.get("idToken").is_none(), "{started}");
}

#[test]
fn given_presented_id_token_without_ev_when_accepted_then_connection_timeout_scheduled_per_e03_fr_05()
 {
    let mut run = Run::accepted(300);
    swipe(&mut run, 0, "TAG1");

    accept(&mut run, 0);

    assert_eq!(
        run.station.next_deadline(),
        Some(MINUTE),
        "EVConnectionTimeOut 60 s"
    );
}

#[test]
fn given_authorized_id_token_without_ev_when_connection_timeout_expires_then_ended_by_timeout_per_e03_fr_05()
 {
    let mut run = Run::accepted(300);
    swipe(&mut run, 0, "TAG1");
    accept(&mut run, 0);

    run.input(MINUTE, Input::Tick);
    run.answer_all(MINUTE);

    let ended = events(&run).pop().unwrap();
    assert_eq!(summary(&ended), s("Ended", "EVConnectTimeout", json!(null)));
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "Timeout");
}

#[test]
fn given_charging_ev_when_its_battery_is_full_then_suspended_by_the_ev_per_e01_fr_19() {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "SampledDataCtrlr.TxUpdatedInterval", "0");
    charging(&mut run, 0);
    let full = run.station.next_deadline().unwrap();

    run.input(full, Input::Tick);
    run.answer_all(full);

    assert_eq!(
        summaries(&run).pop().unwrap(),
        s("Updated", "ChargingStateChanged", json!("SuspendedEV"))
    );
}

#[test]
fn given_presented_id_token_when_blocked_then_no_transaction() {
    let mut run = Run::accepted(300);
    swipe(&mut run, 0, "TAG1");

    run.answer(0, json!({"idTokenInfo": {"status": "Blocked"}}));

    assert!(events(&run).is_empty());
}

#[test]
fn given_finished_transaction_when_ev_plugged_and_unplugged_again_then_new_transaction_id_per_e01_fr_08()
 {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);
    intent(&mut run, 1, Intent::Unplug { connector: 1 });

    plug_in(&mut run, 10_000);
    intent(&mut run, 10_001, Intent::Unplug { connector: 1 });

    let ids: Vec<_> = events(&run)
        .iter()
        .map(|event| event["transactionInfo"]["transactionId"].clone())
        .collect();
    assert_eq!(ids.len(), 4);
    assert_eq!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[2]);
}
