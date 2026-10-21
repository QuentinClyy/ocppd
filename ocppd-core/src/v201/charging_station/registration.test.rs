use crate::api::event::LinkEvent;
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run, boot};
use crate::v201::event::EventKind;
use crate::v201::messages::provisioning::boot::RegistrationStatus;
use serde_json::json;

fn booted_as(status: &str, interval: u32) -> Run {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);
    run.answer(0, boot(status, interval));
    run
}

#[test]
fn given_new_station_when_connected_then_boot_notification_with_power_up_per_b01_fr_01_and_fr_09() {
    let mut run = Run::new(1);

    run.input(0, Input::Connected);

    let (_, action, payload) = run.last_call();
    assert_eq!(action, "BootNotification");
    assert_eq!(
        payload,
        json!({
            "chargingStation": {
                "serialNumber": "SN-1",
                "model": "sim",
                "vendorName": "ocppd",
                "firmwareVersion": "0.1.0"
            },
            "reason": "PowerUp"
        })
    );
}

#[test]
fn given_pending_boot_when_time_passes_then_nothing_else_is_sent_per_b01_fr_08() {
    let mut run = booted_as("Pending", 30);

    run.input(10_000, Input::Tick);

    assert_eq!(run.actions(), vec!["BootNotification"]);
}

#[test]
fn given_two_connectors_when_boot_accepted_then_every_connector_reported_available_per_b01_fr_05() {
    let mut run = Run::new(2);
    run.input(0, Input::Connected);

    run.answer(0, boot("Accepted", 300));
    run.answer_all(0);

    assert!(run.events.contains(&EventKind::BootStatus {
        status: RegistrationStatus::Accepted
    }));
    let statuses: Vec<_> = run
        .calls
        .iter()
        .filter(|(_, action, _)| action == "StatusNotification")
        .collect();
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[1].2["evseId"], 2);
    assert_eq!(statuses[1].2["connectorId"], 1);
    assert_eq!(statuses[1].2["connectorStatus"], "Available");
}

#[test]
fn given_accepted_boot_when_interval_elapses_then_heartbeat_per_b01_fr_04_and_g02_fr_01() {
    let mut run = Run::accepted(300);

    run.input(300_000, Input::Tick);

    assert_eq!(run.actions(), vec!["Heartbeat"]);
}

#[test]
fn given_heartbeat_sent_when_answered_then_next_one_one_interval_later_per_g02_fr_01() {
    let mut run = Run::accepted(300);
    run.input(300_000, Input::Tick);

    run.answer(300_000, json!({"currentTime": "2026-10-21T00:05:00Z"}));

    assert_eq!(run.station.next_deadline(), Some(600_000));
}

#[test]
fn given_pending_or_rejected_boot_when_interval_elapses_then_boot_retried_on_the_same_connection_per_b02_fr_08_and_b03_fr_06()
 {
    for status in ["Pending", "Rejected"] {
        let mut run = booted_as(status, 30);

        run.input(30_000, Input::Tick);

        assert_eq!(
            run.actions(),
            vec!["BootNotification", "BootNotification"],
            "{status}"
        );
        assert!(!run.closed, "{status}: B02.FR.06 keeps the connection");
    }
}

#[test]
fn given_booting_station_when_boot_pending_or_rejected_with_interval_then_retry_due_after_it_per_b02_fr_08_and_b03_fr_06()
 {
    for status in ["Pending", "Rejected"] {
        let mut run = Run::new(1);
        run.input(0, Input::Connected);

        run.answer(0, boot(status, 30));

        assert_eq!(run.station.next_deadline(), Some(30_000), "{status}");
    }
}

#[test]
fn given_booting_station_when_boot_accepted_with_interval_then_heartbeat_due_after_it_per_b01_fr_04()
 {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);

    run.answer(0, boot("Accepted", 300));
    run.answer_all(0);

    assert_eq!(run.station.next_deadline(), Some(300_000));
}

#[test]
fn given_booting_station_when_boot_pending_with_zero_interval_then_retries_on_its_own_interval_per_b02_fr_07()
 {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);

    run.answer(0, boot("Pending", 0));

    assert_eq!(run.station.next_deadline(), Some(MINUTE));
}

#[test]
fn given_booting_station_when_boot_answer_has_custom_data_then_accepted() {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);
    let mut response = boot("Accepted", 300);
    response["customData"] = json!({"vendorId": "example"});

    run.answer(0, response);

    assert!(run.events.contains(&EventKind::BootStatus {
        status: RegistrationStatus::Accepted
    }));
}

#[test]
fn given_booting_station_when_boot_answer_has_unknown_field_then_invalid_and_retried() {
    let mut run = Run::new(1);
    run.input(0, Input::Connected);
    let mut response = boot("Accepted", 300);
    response["extra"] = json!(1);

    run.answer(0, response);

    assert!(matches!(
        run.events.last(),
        Some(EventKind::Link(LinkEvent::InvalidPayload { .. }))
    ));
    assert_eq!(run.station.next_deadline(), Some(MINUTE));
}
