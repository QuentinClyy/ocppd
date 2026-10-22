use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
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
