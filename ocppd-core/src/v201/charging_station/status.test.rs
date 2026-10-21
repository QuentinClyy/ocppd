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
