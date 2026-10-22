use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use crate::v201::event::EventKind;
use crate::v201::messages::availability::ConnectorStatus;

fn statuses(run: &Run) -> Vec<ConnectorStatus> {
    run.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::StatusChanged { status, .. } => Some(*status),
            _ => None,
        })
        .collect()
}

fn evses_reported(run: &Run) -> Vec<serde_json::Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "StatusNotification")
        .map(|(_, _, payload)| payload["evseId"].clone())
        .collect()
}

fn plugged_in(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
}

#[test]
fn given_available_connector_when_ev_plugged_then_occupied_reported_before_the_transaction_per_g01_fr_03_and_tc_g_01()
 {
    let mut run = Run::accepted(300);

    plugged_in(&mut run, 1_000);

    assert_eq!(statuses(&run), vec![ConnectorStatus::Occupied]);
    assert_eq!(
        run.calls[0].1, "StatusNotification",
        "E02 sends the status first"
    );
    assert_eq!(run.calls[0].2["connectorStatus"], "Occupied");
}

#[test]
fn given_occupied_connector_when_ev_unplugged_then_available_reported_per_g01_fr_04_and_tc_g_02() {
    let mut run = Run::accepted(300);
    plugged_in(&mut run, 1_000);
    run.answer(1_000, serde_json::json!({}));

    run.input(2_000, Input::Intent(Intent::Unplug { connector: 1 }));

    assert_eq!(
        statuses(&run),
        vec![ConnectorStatus::Occupied, ConnectorStatus::Available]
    );
}

#[test]
fn given_nothing_changed_when_short_outage_ends_then_nothing_reported_per_b04_fr_02_and_tc_b_52() {
    let mut run = Run::accepted(300);
    run.input(1_000, Input::Disconnected);

    run.input(2_000, Input::Connected);

    assert!(run.actions().is_empty());
}

#[test]
fn given_two_evses_when_long_outage_ends_then_every_connector_reported_without_status_changes_per_b04_fr_01_and_tc_b_51()
 {
    let mut run = Run::accepted_with(2, 300);
    run.input(1_000, Input::Disconnected);

    run.input(62_000, Input::Connected);
    run.answer_all(62_000);

    assert_eq!(evses_reported(&run), vec![1, 2]);
    assert!(statuses(&run).is_empty(), "not status changes");
}

#[test]
fn given_ev_plugged_during_a_short_outage_when_it_ends_then_only_that_connector_reported_per_b04_fr_02()
 {
    let mut run = Run::accepted_with(2, 300);
    run.input(1_000, Input::Disconnected);
    let ev = EvSpec::default();
    run.input(2_000, Input::Intent(Intent::PlugIn { connector: 2, ev }));

    run.input(3_000, Input::Connected);
    run.answer_all(3_000);

    assert_eq!(evses_reported(&run), vec![2]);
}
