use crate::fixtures::v201::remote_control::{events, start};
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn trigger(run: &mut Run, request: Value) -> Value {
    run.result(0, "TriggerMessage", request)["status"].clone()
}

#[test]
fn given_booted_station_when_heartbeat_triggered_then_accepted_and_sent_after_the_response_per_f06_fr_04_and_tc_f_20()
 {
    let mut run = Run::accepted(86_400);

    let status = trigger(&mut run, json!({"requestedMessage": "Heartbeat"}));

    assert_eq!(status, "Accepted");
    assert_eq!(run.last_call().1, "Heartbeat");
}

#[test]
fn given_accepted_boot_when_boot_notification_triggered_then_rejected_per_f06_fr_17_and_tc_f_26() {
    let mut run = Run::accepted(86_400);

    let status = trigger(&mut run, json!({"requestedMessage": "BootNotification"}));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_station_when_status_triggered_without_connector_then_rejected_per_f06_fr_12() {
    let mut run = Run::accepted(86_400);

    let status = trigger(&mut run, json!({"requestedMessage": "StatusNotification"}));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_station_when_status_of_one_connector_triggered_then_accepted_per_f06_fr_12_and_tc_f_23()
 {
    let mut run = Run::accepted(86_400);
    let one =
        json!({"requestedMessage": "StatusNotification", "evse": {"id": 1, "connectorId": 1}});

    let status = trigger(&mut run, one);

    assert_eq!(status, "Accepted");
}

#[test]
fn given_booted_station_when_meter_values_triggered_then_sent_with_trigger_context_per_f06_fr_06_and_tc_f_12()
 {
    let mut run = Run::accepted(86_400);

    let status = trigger(&mut run, json!({"requestedMessage": "MeterValues"}));

    assert_eq!(status, "Accepted");
    let (_, action, payload) = run.last_call();
    assert_eq!(action, "MeterValues");
    assert_eq!(
        payload["meterValue"][0]["sampledValue"][0]["context"],
        "Trigger"
    );
}

#[test]
fn given_no_transaction_when_transaction_event_triggered_then_rejected() {
    let mut run = Run::accepted(86_400);

    let status = trigger(&mut run, json!({"requestedMessage": "TransactionEvent"}));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_ongoing_transaction_when_transaction_event_triggered_then_sent_with_trigger_and_charging_state_per_f06_fr_07_and_tc_f_13()
 {
    let mut run = Run::accepted(86_400);
    start(&mut run, Some(1));

    let status = trigger(&mut run, json!({"requestedMessage": "TransactionEvent"}));
    run.answer_all(0);

    assert_eq!(status, "Accepted");
    let triggered = events(&run).pop().unwrap();
    assert_eq!(triggered["triggerReason"], "Trigger");
    assert!(triggered["transactionInfo"]["chargingState"].is_string());
}

#[test]
fn given_no_iso_15118_when_v2g_certificate_triggered_then_not_implemented_per_f06_fr_08_and_tc_f_27()
 {
    let mut run = Run::accepted(86_400);

    let status = trigger(&mut run, json!({"requestedMessage": "SignV2GCertificate"}));

    assert_eq!(status, "NotImplemented");
}
