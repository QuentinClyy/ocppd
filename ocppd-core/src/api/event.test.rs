use super::*;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::provisioning::boot::RegistrationStatus;
use serde_json::{Value, json};

fn at_5(kind: EventKind) -> Event {
    Event {
        station: "CP1".into(),
        at: 5,
        kind: Kind::V16(kind),
    }
}

#[test]
fn given_boot_status_event_when_serialized_then_tagged_with_its_kind_and_fields() {
    let event = at_5(EventKind::BootStatus {
        status: RegistrationStatus::Accepted,
    });

    let shape = serde_json::to_value(&event).unwrap();

    assert_eq!(
        shape,
        json!({"station": "CP1", "at": 5, "kind": "boot_status", "status": "Accepted"})
    );
}

#[test]
fn given_link_event_when_serialized_then_flattened_into_its_kind() {
    let event = at_5(EventKind::Link(LinkEvent::Connected));

    let shape: Value = serde_json::to_value(&event).unwrap();

    assert_eq!(
        shape,
        json!({"station": "CP1", "at": 5, "kind": "connected"})
    );
}
