use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::{Run, boot};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::provisioning::reset::ResetType;
use serde_json::{Value, json};

fn reset(run: &mut Run, now: u64, request: Value) -> Value {
    run.result(now, "Reset", request)["status"].clone()
}

fn plug_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
}

fn unplug(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(now);
}

fn power_cut(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::PowerCut));
    run.answer_all(now);
}

fn power_returns(run: &mut Run, now: u64) -> Value {
    run.reconnect(now);
    let boot_request = run.boot_request();
    run.answer(now, boot("Accepted", 300));
    run.answer_all(now);
    boot_request
}

fn sent(run: &Run, action: &str) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, sent, _)| sent == action)
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

#[test]
fn given_booted_station_when_reset_immediate_then_accepted_and_reboots_per_b11_fr_03() {
    let mut run = Run::accepted(300);

    let status = reset(&mut run, 0, json!({"type": "Immediate"}));

    assert_eq!(status, "Accepted");
    assert!(run.closed, "reboots once the answer is out");
    assert!(run.events.contains(&EventKind::Reset {
        reset_type: ResetType::Immediate
    }));
}

#[test]
fn given_immediate_reset_when_reconnected_then_boots_with_remote_reset_per_b11_fr_04() {
    let mut run = Run::accepted(300);
    reset(&mut run, 0, json!({"type": "Immediate"}));

    run.reconnect(1_000);

    assert_eq!(run.boot_request()["reason"], "RemoteReset");
}

#[test]
fn given_booted_station_when_reset_of_one_evse_then_rejected_per_b11_fr_09() {
    let mut run = Run::accepted(300);

    let status = reset(&mut run, 0, json!({"type": "OnIdle", "evseId": 1}));

    assert_eq!(status, "Rejected");
    assert!(!run.closed);
}

#[test]
fn given_plugged_ev_when_reset_on_idle_then_scheduled_per_tc_b_21() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);

    let status = reset(&mut run, 0, json!({"type": "OnIdle"}));

    assert_eq!(status, "Scheduled");
    assert!(!run.closed);
}

#[test]
fn given_scheduled_reset_when_ev_unplugged_then_reboots_per_tc_b_21() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);
    reset(&mut run, 0, json!({"type": "OnIdle"}));

    unplug(&mut run, 1_000);

    assert!(run.closed, "reboots once idle");
}

#[test]
fn given_scheduled_reset_done_when_reconnected_then_boots_with_scheduled_reset_per_tc_b_21() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);
    reset(&mut run, 0, json!({"type": "OnIdle"}));
    unplug(&mut run, 1_000);

    run.reconnect(2_000);

    assert_eq!(run.boot_request()["reason"], "ScheduledReset");
}

#[test]
fn given_plugged_ev_when_power_cut_then_transaction_ends_by_power_loss_and_station_goes_down() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);

    power_cut(&mut run, 1_000);

    assert!(run.closed, "down once the outbox is drained");
    let ended = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(ended["eventType"], "Ended");
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "PowerLoss");
}

#[test]
fn given_power_cut_when_power_returns_then_boots_with_power_up_and_reports_startup_only() {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);
    power_cut(&mut run, 1_000);

    let boot_request = power_returns(&mut run, 2_000);

    assert_eq!(boot_request["reason"], "PowerUp");
    let types: Vec<_> = sent(&run, "SecurityEventNotification")
        .into_iter()
        .map(|payload| payload["type"].clone())
        .collect();
    assert!(!types.contains(&json!("ResetOrReboot")), "{types:?}");
    assert!(types.contains(&json!("StartupOfTheDevice")), "{types:?}");
}

#[test]
fn given_inoperative_station_cut_from_power_when_power_returns_then_still_unavailable() {
    let mut run = Run::accepted(300);
    let inoperative = json!({"operationalStatus": "Inoperative"});
    run.result(0, "ChangeAvailability", inoperative);
    run.answer_all(0);
    power_cut(&mut run, 1_000);

    power_returns(&mut run, 2_000);

    let last = sent(&run, "StatusNotification").pop().unwrap();
    assert_eq!(last["connectorStatus"], "Unavailable");
}

#[test]
fn given_power_came_back_and_reset_immediate_when_reconnected_then_boots_with_remote_reset() {
    let mut run = Run::accepted(300);
    power_cut(&mut run, 1_000);
    power_returns(&mut run, 2_000);
    reset(&mut run, 5_000, json!({"type": "Immediate"}));

    run.reconnect(6_000);

    assert_eq!(run.boot_request()["reason"], "RemoteReset");
}

#[test]
fn given_plugged_ev_when_shut_down_then_the_transaction_ends_as_other_and_the_station_finishes_once_it_is_answered()
 {
    let mut run = Run::accepted(300);
    plug_in(&mut run, 0);

    run.input(1_000, Input::Intent(Intent::Shutdown));
    let waiting = run.station.finished(1_000);
    run.answer_all(1_000);

    let ended = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(ended["eventType"], "Ended");
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "Other");
    assert!(!waiting, "not finished while the event is unanswered");
    assert!(run.station.finished(1_000));
}
