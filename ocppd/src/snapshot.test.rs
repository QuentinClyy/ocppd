use super::*;

fn snapshot(lines: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    for line in lines {
        snapshot.apply(line);
    }
    snapshot
}

const CONNECTED: &str = r#"{"station":"CP1","at":1,"kind":"connected"}"#;
const AVAILABLE: &str =
    r#"{"station":"CP1","at":2,"kind":"status_changed","connector":1,"status":"Available"}"#;
const CHARGING: &str =
    r#"{"station":"CP1","at":5,"kind":"status_changed","connector":1,"status":"Charging"}"#;
const OTHER_CONNECTOR: &str =
    r#"{"station":"CP1","at":3,"kind":"status_changed","connector":2,"status":"Available"}"#;
const STARTED: &str =
    r#"{"station":"CP1","at":4,"kind":"transaction_started","connector":1,"transaction_id":7}"#;
const STOPPED: &str = r#"{"station":"CP1","at":6,"kind":"transaction_stopped","connector":1,"transaction_id":7,"reason":"Local"}"#;
const OTHER_STATION: &str = r#"{"station":"CP2","at":2,"kind":"connected"}"#;

#[test]
fn given_a_status_that_changes_twice_when_snapshotted_then_only_the_latest_per_connector() {
    let snapshot = snapshot(&[AVAILABLE, OTHER_CONNECTOR, CHARGING]);

    let lines = snapshot.lines();

    assert_eq!(lines, [OTHER_CONNECTOR, CHARGING]);
}

#[test]
fn given_a_started_then_stopped_transaction_when_snapshotted_then_both_in_time_order() {
    let snapshot = snapshot(&[CONNECTED, STARTED, STOPPED]);

    let lines = snapshot.lines();

    assert_eq!(lines, [CONNECTED, STARTED, STOPPED]);
}

#[test]
fn given_events_from_two_stations_when_one_is_forgotten_then_only_the_other_remains() {
    let mut snapshot = snapshot(&[CONNECTED, OTHER_STATION, AVAILABLE]);

    snapshot.forget("CP1");

    assert_eq!(snapshot.lines(), [OTHER_STATION]);
}

#[test]
fn given_a_line_without_a_station_when_applied_then_it_is_not_kept() {
    let rejected = r#"{"at":1,"kind":"command_rejected","reason":"expected ident"}"#;

    let snapshot = snapshot(&[rejected, "not json"]);

    assert!(snapshot.lines().is_empty());
}

#[test]
fn given_two_events_at_the_same_millisecond_when_snapshotted_then_in_arrival_order() {
    let boot = r#"{"station":"CP1","at":1,"kind":"boot_status","status":"Accepted"}"#;

    let snapshot = snapshot(&[boot, CONNECTED]);

    assert_eq!(snapshot.lines(), [boot, CONNECTED]);
}
