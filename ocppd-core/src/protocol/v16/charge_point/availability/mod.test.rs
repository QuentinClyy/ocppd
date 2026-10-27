use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::fixtures::v16::replies::status;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use serde_json::json;

fn inoperative_connector() -> Csms {
    let mut csms = Csms::booted();
    let inoperative = json!({"connectorId": 1, "type": "Inoperative"});
    csms.call(0, "ChangeAvailability", inoperative);
    csms
}

fn plug_in(csms: &mut Csms) {
    csms.intent(
        0,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
}

fn connector_1_statuses(csms: &Csms) -> Vec<ChargePointStatus> {
    csms.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::StatusChanged {
                connector: 1,
                status,
            } => Some(*status),
            _ => None,
        })
        .collect()
}

fn scheduled_inoperative_station() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    let whole_station = json!({"connectorId": 0, "type": "Inoperative"});
    csms.call(MINUTE, "ChangeAvailability", whole_station);
    csms
}

#[test]
fn given_booted_charge_point_when_connector_made_inoperative_then_accepted_and_unavailable() {
    let mut csms = Csms::booted();
    let inoperative = json!({"connectorId": 1, "type": "Inoperative"});

    let reply = csms.call(0, "ChangeAvailability", inoperative);

    assert_eq!(status(reply), "Accepted");
    assert_eq!(csms.statuses(), vec![ChargePointStatus::Unavailable]);
}

#[test]
fn given_inoperative_connector_with_plugged_ev_when_id_tag_presented_then_refused_as_unavailable() {
    let mut csms = inoperative_connector();
    plug_in(&mut csms);
    csms.events.clear();

    csms.intent(0, authorize("TAG1"));

    let rejected = EventKind::Link(LinkEvent::IntentRejected {
        connector: Some(1),
        reason: "unavailable".into(),
    });
    assert_eq!(csms.events, vec![rejected]);
}

#[test]
fn given_inoperative_connector_when_remote_start_then_rejected() {
    let mut csms = inoperative_connector();
    let remote = json!({"connectorId": 1, "idTag": "R1"});

    let reply = csms.call(0, "RemoteStartTransaction", remote);

    assert_eq!(status(reply), "Rejected");
}

#[test]
fn given_inoperative_connector_with_plugged_ev_when_made_operative_then_accepted_and_preparing() {
    let mut csms = inoperative_connector();
    plug_in(&mut csms);
    let operative = json!({"connectorId": 1, "type": "Operative"});

    let reply = csms.call(0, "ChangeAvailability", operative);

    assert_eq!(status(reply), "Accepted");
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Preparing));
}

#[test]
fn given_charging_ev_when_station_made_inoperative_then_scheduled_station_unavailable_and_connector_still_charging()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    let whole_station = json!({"connectorId": 0, "type": "Inoperative"});

    let reply = csms.call(MINUTE, "ChangeAvailability", whole_station);

    assert_eq!(status(reply), "Scheduled");
    assert_eq!(
        csms.statuses().last(),
        Some(&ChargePointStatus::Unavailable)
    );
    assert_eq!(
        connector_1_statuses(&csms).last(),
        Some(&ChargePointStatus::Charging)
    );
}

#[test]
fn given_scheduled_inoperative_station_when_transaction_stopped_then_unavailable() {
    let mut csms = scheduled_inoperative_station();

    csms.intent(2 * MINUTE, Intent::Stop { connector: 1 });

    assert_eq!(
        csms.statuses().last(),
        Some(&ChargePointStatus::Unavailable)
    );
}
