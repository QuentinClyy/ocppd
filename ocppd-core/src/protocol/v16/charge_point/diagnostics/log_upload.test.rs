use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::result;
use crate::protocol::shared::file_transfer::TransferStage;
use serde_json::{Value, json};

fn get_log(csms: &mut Csms, now: u64, request_id: i32, location: &str) -> Value {
    let request = json!({
        "log": {"remoteLocation": location},
        "logType": "SecurityLog",
        "requestId": request_id,
    });
    result(csms.call(now, "GetLog", request))
}

fn statuses(csms: &Csms) -> Vec<Value> {
    csms.sent("LogStatusNotification")
}

fn uploading() -> Csms {
    let mut csms = Csms::booted();
    get_log(&mut csms, 0, 7, "https://logs.example/");
    csms
}

fn trigger_log_status(csms: &mut Csms, now: u64) {
    let trigger = json!({"requestedMessage": "LogStatusNotification"});
    csms.call(now, "ExtendedTriggerMessage", trigger);
}

#[test]
fn given_booted_charge_point_when_security_log_requested_then_accepted_with_its_filename_and_due_in_10_seconds()
 {
    let mut csms = Csms::booted();

    let reply = get_log(&mut csms, 0, 7, "https://logs.example/");

    assert_eq!(
        reply,
        json!({"status": "Accepted", "filename": "CP1-security.log"})
    );
    assert_eq!(csms.charge_point.next_deadline(), Some(10_000));
}

#[test]
fn given_log_upload_when_it_ends_then_uploading_then_uploaded_under_its_request_id() {
    let mut csms = uploading();

    csms.input(10_000, Input::Tick);

    assert_eq!(
        statuses(&csms),
        vec![
            json!({"status": "Uploading", "requestId": 7}),
            json!({"status": "Uploaded", "requestId": 7}),
        ]
    );
}

#[test]
fn given_log_upload_when_a_new_one_requested_then_accepted_canceled_and_restarted() {
    let mut csms = uploading();

    let reply = get_log(&mut csms, 5_000, 8, "https://logs.example/");

    assert_eq!(reply["status"], "AcceptedCanceled");
    assert_eq!(csms.charge_point.next_deadline(), Some(15_000));
}

#[test]
fn given_booted_charge_point_when_log_requested_to_an_unsupported_location_then_upload_failure() {
    let mut csms = Csms::booted();

    get_log(&mut csms, 0, 7, "smb://logs/");

    assert_eq!(
        statuses(&csms),
        vec![json!({"status": "UploadFailure", "requestId": 7})]
    );
}

#[test]
fn given_no_upload_when_log_status_triggered_then_idle() {
    let mut csms = Csms::booted();

    trigger_log_status(&mut csms, 0);

    assert_eq!(statuses(&csms), vec![json!({"status": "Idle"})]);
}

#[test]
fn given_log_upload_when_log_status_triggered_then_uploading_reported_again() {
    let mut csms = uploading();

    trigger_log_status(&mut csms, 1_000);

    assert_eq!(
        statuses(&csms),
        vec![
            json!({"status": "Uploading", "requestId": 7}),
            json!({"status": "Uploading", "requestId": 7}),
        ]
    );
}

#[test]
fn given_failing_upload_when_the_log_upload_ends_then_upload_failure() {
    let mut csms = Csms::booted();
    let stage = TransferStage::Upload;
    csms.intent(0, Intent::FailTransfer { stage });
    get_log(&mut csms, 0, 7, "https://logs.example/");

    csms.input(10_000, Input::Tick);

    assert_eq!(
        statuses(&csms)[1],
        json!({"status": "UploadFailure", "requestId": 7})
    );
}
