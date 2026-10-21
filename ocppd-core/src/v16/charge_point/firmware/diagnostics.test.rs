use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::Csms;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn get_diagnostics(csms: &mut Csms, location: &str) -> Value {
    match csms.call(0, "GetDiagnostics", json!({"location": location})) {
        Frame::CallResult { payload, .. } => payload,
        other => panic!("expected a result, got {other:?}"),
    }
}

fn statuses(csms: &Csms) -> Vec<Value> {
    let sent = csms.sent("DiagnosticsStatusNotification");
    sent.iter().map(|sent| sent["status"].clone()).collect()
}

#[test]
fn given_booted_charge_point_when_diagnostics_requested_then_file_name_returned_and_uploading() {
    let mut csms = Csms::booted();

    let reply = get_diagnostics(&mut csms, "ftp://logs.example/upload/");

    assert_eq!(reply, json!({"fileName": "CP1-diagnostics.log"}));
    assert_eq!(statuses(&csms), vec![json!("Uploading")]);
    assert_eq!(csms.charge_point.next_deadline(), Some(10_000));
}

#[test]
fn given_uploading_diagnostics_when_the_upload_ends_then_uploaded() {
    let mut csms = Csms::booted();
    get_diagnostics(&mut csms, "ftp://logs.example/upload/");

    csms.input(10_000, Input::Tick);

    assert_eq!(statuses(&csms), vec![json!("Uploading"), json!("Uploaded")]);
}

#[test]
fn given_booted_charge_point_when_diagnostics_requested_to_an_smb_location_then_upload_failed() {
    let mut csms = Csms::booted();

    get_diagnostics(&mut csms, "smb://logs/");

    assert_eq!(statuses(&csms), vec![json!("UploadFailed")]);
}

#[test]
fn given_uploading_diagnostics_when_their_status_triggered_then_uploading_reported_again() {
    let mut csms = Csms::booted();
    get_diagnostics(&mut csms, "https://logs.example/");
    let request = json!({"requestedMessage": "DiagnosticsStatusNotification"});

    csms.call(1_000, "TriggerMessage", request);

    assert_eq!(
        statuses(&csms),
        vec![json!("Uploading"), json!("Uploading")]
    );
}
