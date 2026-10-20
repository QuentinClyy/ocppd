use crate::api::intent::EvSpec;
use crate::api::io::{Input, Output};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, register};
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn trigger(csms: &mut Csms, now: u64, request: Value) -> Value {
    match csms.call(now, "TriggerMessage", request) {
        Frame::CallResult { payload, .. } => payload["status"].clone(),
        other => panic!("expected a result, got {other:?}"),
    }
}

#[test]
fn given_booted_charge_point_when_heartbeat_triggered_then_the_confirmation_goes_out_before_the_heartbeat()
 {
    let mut csms = Csms::booted();
    let call = json!([2, "t1", "TriggerMessage", {"requestedMessage": "Heartbeat"}]);

    let outputs = csms.charge_point.handle(0, Input::Frame(call.to_string()));

    let sent: Vec<_> = outputs
        .into_iter()
        .filter_map(|output| match output {
            Output::Send(frame) => Some(frame),
            _ => None,
        })
        .collect();
    assert!(matches!(&sent[..], [
        Frame::CallResult { id, .. },
        Frame::Call { action, .. },
    ] if id == "t1" && action == "Heartbeat"));
}

#[test]
fn given_booted_charge_point_when_boot_notification_triggered_then_accepted_and_sent() {
    let mut csms = Csms::booted();

    let status = trigger(
        &mut csms,
        0,
        json!({"requestedMessage": "BootNotification"}),
    );

    assert_eq!(status, "Accepted");
    assert_eq!(csms.expect("BootNotification")["chargePointModel"], "sim");
}

#[test]
fn given_two_connectors_when_status_of_connector_2_triggered_then_only_its_status_sent() {
    let mut csms = Csms::booted_with(2);
    let request = json!({"requestedMessage": "StatusNotification", "connectorId": 2});

    let status = trigger(&mut csms, 0, request);

    assert_eq!(status, "Accepted");
    let sent = csms.sent("StatusNotification");
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["connectorId"], 2);
    assert_eq!(sent[0]["status"], "Available");
}

#[test]
fn given_two_connectors_when_status_triggered_without_connector_then_every_status_sent() {
    let mut csms = Csms::booted_with(2);

    trigger(
        &mut csms,
        0,
        json!({"requestedMessage": "StatusNotification"}),
    );

    let connectors: Vec<_> = csms
        .sent("StatusNotification")
        .iter()
        .map(|sent| sent["connectorId"].clone())
        .collect();
    assert_eq!(connectors, vec![json!(0), json!(1), json!(2)]);
}

#[test]
fn given_charging_ev_when_meter_values_triggered_then_sent_with_the_transaction_and_trigger_context()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    let request = json!({"requestedMessage": "MeterValues", "connectorId": 1});

    let status = trigger(&mut csms, MINUTE / 2, request);

    assert_eq!(status, "Accepted");
    let sent = &csms.sent("MeterValues")[0];
    assert_eq!(sent["transactionId"], 42);
    assert_eq!(sent["meterValue"][0]["timestamp"], "1970-01-01T00:00:30Z");
    assert_eq!(
        sent["meterValue"][0]["sampledValue"][0]["context"],
        "Trigger"
    );
    assert_eq!(register(sent), 92.0);
}

#[test]
fn given_idle_connector_when_meter_values_triggered_then_sent_without_transaction() {
    let mut csms = Csms::booted();

    trigger(&mut csms, 0, json!({"requestedMessage": "MeterValues"}));

    let sent = &csms.sent("MeterValues")[0];
    assert_eq!(sent["connectorId"], 1);
    assert!(sent.get("transactionId").is_none());
    assert_eq!(register(sent), 0.0);
}

#[test]
fn given_no_update_when_firmware_status_triggered_then_idle() {
    let mut csms = Csms::booted();

    trigger(
        &mut csms,
        0,
        json!({"requestedMessage": "FirmwareStatusNotification"}),
    );

    assert_eq!(
        csms.sent("FirmwareStatusNotification"),
        vec![json!({"status": "Idle"})]
    );
}

#[test]
fn given_no_upload_when_diagnostics_status_triggered_then_idle() {
    let mut csms = Csms::booted();

    trigger(
        &mut csms,
        0,
        json!({"requestedMessage": "DiagnosticsStatusNotification"}),
    );

    assert_eq!(
        csms.sent("DiagnosticsStatusNotification"),
        vec![json!({"status": "Idle"})]
    );
}

#[test]
fn given_one_connector_when_status_of_connector_2_triggered_then_rejected_and_nothing_sent() {
    let mut csms = Csms::booted();
    let request = json!({"requestedMessage": "StatusNotification", "connectorId": 2});

    let status = trigger(&mut csms, 0, request);

    assert_eq!(status, "Rejected");
    assert!(csms.sent.is_empty());
}

#[test]
fn given_no_main_meter_when_meter_values_of_connector_0_triggered_then_rejected() {
    let mut csms = Csms::booted();
    let main_meter = json!({"requestedMessage": "MeterValues", "connectorId": 0});

    let status = trigger(&mut csms, 0, main_meter);

    assert_eq!(status, "Rejected");
}
