use crate::api::event::EventKind;
use crate::api::io::{Input, Output};
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::raw::{
    boot_response, calls, charge_point, kinds, only_call, result, station,
};
use crate::station::Station;
use crate::v16::charge_point::ChargePoint;
use crate::v16::messages::core::boot::RegistrationStatus;
use crate::v16::messages::core::status::ChargePointStatus;
use serde_json::json;
use std::collections::BTreeSet;

fn booted_and_reported(now: u64) -> (ChargePoint, Vec<Output>) {
    let mut charge_point = charge_point();
    let (boot_id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let outputs = charge_point.handle(now, result(&boot_id, boot_response("Accepted", 300)));
    (charge_point, outputs)
}

fn drained(charge_point: &mut ChargePoint, now: u64, mut outputs: Vec<Output>) {
    while let Some((id, _, _)) = calls(&outputs).pop() {
        outputs = charge_point.handle(now, result(&id, json!({})));
    }
}

fn idle() -> ChargePoint {
    let (mut charge_point, outputs) = booted_and_reported(1_000);
    drained(&mut charge_point, 1_000, outputs);
    charge_point
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

fn fleet(jitter: bool) -> BTreeSet<u64> {
    (1..=20)
        .map(|n| first_heartbeat(&format!("CP{n}"), jitter))
        .collect()
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

fn answered_boot(status: &str) -> (ChargePoint, String, Vec<Output>) {
    let mut charge_point = charge_point();
    let (first_id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    let outputs = charge_point.handle(1_000, result(&first_id, boot_response(status, 60)));
    (charge_point, first_id, outputs)
}

fn boot_sent() -> (ChargePoint, String) {
    let mut charge_point = charge_point();
    let (id, _, _) = only_call(&charge_point.handle(0, Input::Connected));
    (charge_point, id)
}

#[test]
fn given_new_charge_point_when_connected_then_boot_notification_and_startup_event() {
    let mut charge_point = charge_point();

    let outputs = charge_point.handle(0, Input::Connected);

    let (_, action, payload) = only_call(&outputs);
    assert_eq!(action, "BootNotification");
    assert_eq!(
        payload,
        json!({
            "chargePointVendor": "ocppd",
            "chargePointModel": "sim",
            "chargePointSerialNumber": "SN-1",
            "firmwareVersion": "0.1.0"
        })
    );
    let startup = EventKind::SecurityEvent {
        event_type: "StartupOfTheDevice".into(),
    };
    assert_eq!(kinds(&outputs), vec![EventKind::Connected, startup]);
}

#[test]
fn given_vendor_name_over_20_characters_when_charge_point_created_then_refused() {
    let station = Station {
        vendor: "a vendor name over twenty".into(),
        ..station()
    };

    let error = ChargePoint::new(station).err().unwrap();

    assert_eq!(error.max, 20);
}

#[test]
fn given_sent_boot_when_accepted_then_every_connector_available_and_startup_event_sent_first() {
    let (_, outputs) = booted_and_reported(1_000);

    assert_eq!(
        kinds(&outputs),
        vec![
            EventKind::BootStatus {
                status: RegistrationStatus::Accepted
            },
            EventKind::StatusChanged {
                connector: 0,
                status: ChargePointStatus::Available
            },
            EventKind::StatusChanged {
                connector: 1,
                status: ChargePointStatus::Available
            }
        ]
    );
    let (_, action, payload) = only_call(&outputs);
    assert_eq!(action, "SecurityEventNotification");
    assert_eq!(payload["type"], "StartupOfTheDevice");
}

#[test]
fn given_accepted_boot_when_startup_event_answered_then_status_of_connector_0_sent() {
    let (mut charge_point, outputs) = booted_and_reported(1_000);
    let (startup_id, _, _) = only_call(&outputs);

    let outputs = charge_point.handle(1_000, result(&startup_id, json!({})));

    let (_, action, payload) = only_call(&outputs);
    assert_eq!(action, "StatusNotification");
    assert_eq!(
        payload,
        json!({"connectorId": 0, "errorCode": "NoError", "status": "Available"})
    );
}

#[test]
fn given_status_of_connector_0_sent_when_answered_then_status_of_connector_1_sent_one_at_a_time() {
    let (mut charge_point, outputs) = booted_and_reported(1_000);
    let (startup_id, _, _) = only_call(&outputs);
    let outputs = charge_point.handle(1_000, result(&startup_id, json!({})));
    let (first_id, _, _) = only_call(&outputs);

    let outputs = charge_point.handle(1_000, result(&first_id, json!({})));

    let (second_id, _, payload) = only_call(&outputs);
    assert_eq!(payload["connectorId"], 1);
    assert!(calls(&charge_point.handle(1_000, result(&second_id, json!({})))).is_empty());
}

#[test]
fn given_sent_boot_when_rejected_or_pending_then_nothing_sent_and_retry_after_the_interval() {
    for status in ["Rejected", "Pending"] {
        let (charge_point, _, outputs) = answered_boot(status);

        let deadline = charge_point.next_deadline();

        assert!(calls(&outputs).is_empty(), "{status}");
        assert_eq!(deadline, Some(61_000), "{status}");
    }
}

#[test]
fn given_rejected_or_pending_boot_when_just_before_the_interval_then_nothing_sent() {
    for status in ["Rejected", "Pending"] {
        let (mut charge_point, _, _) = answered_boot(status);

        let outputs = charge_point.handle(60_999, Input::Tick);

        assert!(calls(&outputs).is_empty(), "{status}");
    }
}

#[test]
fn given_rejected_or_pending_boot_when_the_interval_passes_then_new_boot_notification() {
    for status in ["Rejected", "Pending"] {
        let (mut charge_point, first_id, _) = answered_boot(status);

        let outputs = charge_point.handle(61_000, Input::Tick);

        let (retry_id, action, _) = only_call(&outputs);
        assert_eq!(action, "BootNotification", "{status}");
        assert_ne!(retry_id, first_id);
    }
}

#[test]
fn given_sent_boot_when_never_answered_then_timeout_due_after_30_s() {
    let (charge_point, _) = boot_sent();

    let deadline = charge_point.next_deadline();

    assert_eq!(deadline, Some(30_000));
}

#[test]
fn given_unanswered_boot_when_it_times_out_then_reported_and_retried_a_minute_later() {
    let (mut charge_point, _) = boot_sent();

    let outputs = charge_point.handle(30_000, Input::Tick);

    assert!(calls(&outputs).is_empty());
    assert_eq!(
        kinds(&outputs),
        vec![EventKind::CallTimedOut {
            action: "BootNotification".into()
        }]
    );
    assert_eq!(charge_point.next_deadline(), Some(90_000));
}

#[test]
fn given_timed_out_boot_when_its_retry_is_due_then_boot_notification_again() {
    let (mut charge_point, _) = boot_sent();
    charge_point.handle(30_000, Input::Tick);

    let outputs = charge_point.handle(90_000, Input::Tick);

    let (_, action, _) = only_call(&outputs);
    assert_eq!(action, "BootNotification");
}

#[test]
fn given_sent_boot_when_answered_with_call_error_then_reported_and_retried_later() {
    let (mut charge_point, id) = boot_sent();
    let error = json!([4, id, "InternalError", "", {}]).to_string();

    let outputs = charge_point.handle(1_000, Input::Frame(error));

    assert_eq!(
        kinds(&outputs),
        vec![EventKind::CallFailed {
            action: "BootNotification".into(),
            code: "InternalError".into()
        }]
    );
    assert_eq!(charge_point.next_deadline(), Some(61_000));
}

#[test]
fn given_sent_boot_when_answer_has_an_unknown_field_then_invalid_and_retried_later() {
    let (mut charge_point, id) = boot_sent();
    let response = json!({
        "status": "Accepted",
        "currentTime": "2026-10-20T00:00:00Z",
        "interval": 300,
        "extra": 1
    });

    let outputs = charge_point.handle(1_000, result(&id, response));

    assert!(matches!(
        kinds(&outputs)[..],
        [EventKind::InvalidPayload { .. }]
    ));
    assert_eq!(charge_point.next_deadline(), Some(61_000));
}

#[test]
fn given_sent_boot_when_answer_cannot_be_decoded_then_invalid_and_retried_later() {
    let (mut charge_point, id) = boot_sent();

    let outputs = charge_point.handle(1_000, result(&id, json!({"status": "Maybe"})));

    assert!(matches!(
        kinds(&outputs)[..],
        [EventKind::InvalidPayload { .. }]
    ));
    assert_eq!(charge_point.next_deadline(), Some(61_000));
}

#[test]
fn given_booted_charge_point_when_reconnected_then_statuses_resent_without_status_changes() {
    let mut csms = Csms::booted();
    csms.disconnect(1_000);

    csms.reconnect(2_000);

    assert_eq!(csms.sent("StatusNotification").len(), 2);
    assert!(csms.statuses().is_empty());
}
