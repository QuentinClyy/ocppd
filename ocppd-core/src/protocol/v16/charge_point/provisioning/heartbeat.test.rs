use crate::api::io::Input;
use crate::fixtures::v16::raw::{
    boot_response, booted_and_reported, calls, drained, only_call, result, station,
};
use crate::protocol::v16::charge_point::ChargePoint;
use crate::station::Station;
use serde_json::json;
use std::collections::BTreeSet;

fn idle() -> ChargePoint {
    let (mut charge_point, outputs) = booted_and_reported(1_000);
    drained(&mut charge_point, 1_000, outputs);
    charge_point
}

fn fleet(jitter: bool) -> BTreeSet<u64> {
    (1..=20)
        .map(|n| first_heartbeat(&format!("CP{n}"), jitter))
        .collect()
}

fn first_heartbeat(id: &str, jitter: bool) -> u64 {
    let station = Station {
        id: id.into(),
        jitter,
        ..station()
    };
    let mut charge_point = ChargePoint::new(station).unwrap();
    let (boot_id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let accepted = boot_response("Accepted", 300);
    let outputs = charge_point.handle(1_000_000, result(&boot_id, accepted));
    drained(&mut charge_point, 1_000_000, outputs);
    charge_point.next_deadline().unwrap()
}

#[test]
fn given_idle_charge_point_when_just_before_the_first_heartbeat_then_nothing_sent() {
    let mut charge_point = idle();
    let first = charge_point.next_deadline().unwrap();

    let outputs = charge_point.handle(first - 1, Input::Tick);

    assert!(
        (1_000..=301_000).contains(&first),
        "first heartbeat at {first}"
    );
    assert!(calls(&outputs).is_empty());
}

#[test]
fn given_idle_charge_point_when_the_first_heartbeat_is_due_then_empty_heartbeat_sent() {
    let mut charge_point = idle();
    let first = charge_point.next_deadline().unwrap();

    let outputs = charge_point.handle(first, Input::Tick);

    let (_, action, payload) = only_call(&outputs);
    assert_eq!(action, "Heartbeat");
    assert_eq!(payload, json!({}));
}

#[test]
fn given_sent_heartbeat_when_answered_then_next_one_an_interval_later() {
    let mut charge_point = idle();
    let first = charge_point.next_deadline().unwrap();
    let (heartbeat_id, _, _) = only_call(&charge_point.handle(first, Input::Tick));

    charge_point.handle(
        first + 500,
        result(&heartbeat_id, json!({"currentTime": "x"})),
    );

    assert_eq!(charge_point.next_deadline(), Some(first + 300_000));
}

#[test]
fn given_fleet_with_jitter_when_booted_together_then_first_heartbeats_spread_over_the_interval() {
    let spread = fleet(true);

    assert!(spread.len() > 15, "{spread:?}");
    assert!(
        spread
            .iter()
            .all(|&at| (1_000_000..=1_300_000).contains(&at))
    );
}

#[test]
fn given_fleet_without_jitter_when_booted_together_then_first_heartbeats_aligned() {
    let aligned = fleet(false);

    assert_eq!(aligned, BTreeSet::from([1_300_000]));
}
