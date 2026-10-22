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

#[test]
fn given_plugged_ev_when_id_token_presented_then_authorize_sent_per_e02_fr_06() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 1_000);

    swipe(&mut run, 2_000, "TAG1");

    assert_eq!(run.last_call().1, "Authorize");
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
