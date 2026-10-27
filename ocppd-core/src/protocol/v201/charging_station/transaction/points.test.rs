use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
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
