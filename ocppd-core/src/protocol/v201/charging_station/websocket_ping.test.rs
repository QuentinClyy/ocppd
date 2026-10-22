use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::json;

fn ping_every(run: &mut Run, seconds: &str) {
    let data = json!({"component": {"name": "OCPPCommCtrlr"}, "variable": {"name": "WebSocketPingInterval"}, "attributeValue": seconds});
    let response = run.result(0, "SetVariables", json!({"setVariableData": [data]}));
    assert_eq!(
        response["setVariableResult"][0]["attributeStatus"],
        "Accepted"
    );
}

#[test]
fn given_default_configuration_when_time_passes_then_no_ping() {
    let mut run = Run::accepted(86_400);

    run.input(10 * MINUTE, Input::Tick);

    assert_eq!(run.pings, 0);
}

#[test]
fn given_booted_station_when_ping_interval_set_then_first_ping_scheduled() {
    let mut run = Run::accepted(86_400);

    ping_every(&mut run, "30");

    assert_eq!(run.station.next_deadline(), Some(30_000));
}

#[test]
fn given_ping_interval_when_it_passes_then_pinged_and_next_ping_scheduled() {
    let mut run = Run::accepted(86_400);
    ping_every(&mut run, "30");

    run.input(30_000, Input::Tick);

    assert_eq!(run.pings, 1);
    assert_eq!(run.station.next_deadline(), Some(MINUTE));
}

#[test]
fn given_one_ping_sent_when_the_interval_passes_again_then_pinged_again() {
    let mut run = Run::accepted(86_400);
    ping_every(&mut run, "30");
    run.input(30_000, Input::Tick);

    run.input(MINUTE, Input::Tick);

    assert_eq!(run.pings, 2);
}

#[test]
fn given_ping_disabled_again_when_time_passes_then_no_ping() {
    let mut run = Run::accepted(86_400);
    ping_every(&mut run, "30");
    ping_every(&mut run, "0");

    run.input(MINUTE, Input::Tick);

    assert_eq!(run.pings, 0);
}

#[test]
fn given_ping_interval_while_offline_when_time_passes_then_no_ping() {
    let mut run = Run::accepted(86_400);
    ping_every(&mut run, "30");
    run.input(MINUTE, Input::Disconnected);

    run.input(2 * MINUTE, Input::Tick);

    assert_eq!(run.pings, 0);
}
