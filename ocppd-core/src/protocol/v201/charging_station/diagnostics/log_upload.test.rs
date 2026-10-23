use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use crate::protocol::shared::file_transfer::TransferStage;
use serde_json::{Value, json};

fn get_log(run: &mut Run, request_id: i32, location: &str) -> Value {
    let request = json!({"logType": "SecurityLog", "requestId": request_id, "log": {"remoteLocation": location}});
    let reply = run.result(0, "GetLog", request);
    run.answer_all(0);
    reply
}

fn notifications(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "LogStatusNotification")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

fn trigger_status(run: &mut Run) -> Value {
    let trigger = json!({"requestedMessage": "LogStatusNotification"});
    let status = run.result(0, "TriggerMessage", trigger)["status"].clone();
    run.answer_all(0);
    status
}

fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

fn failing_with_3_retries_of_a_minute() -> Run {
    let mut run = Run::accepted(86_400);
    let stage = TransferStage::Upload;
    run.input(0, Input::Intent(Intent::FailTransfer { stage }));
    let request = json!({"logType": "DiagnosticsLog", "requestId": 4, "retries": 3, "retryInterval": 60, "log": {"remoteLocation": "https://logs.example/missing/"}});
    run.result(0, "GetLog", request);
    run.answer_all(0);
    run
}

#[test]
fn given_failing_upload_with_3_retries_when_just_before_the_last_attempt_ends_then_only_uploading_per_n01_fr_10()
 {
    let mut run = failing_with_3_retries_of_a_minute();

    for now in [10_000, 70_000, 130_000, 189_999] {
        tick(&mut run, now);
    }

    assert_eq!(
        notifications(&run),
        vec![json!({"status": "Uploading", "requestId": 4})]
    );
}

#[test]
fn given_failing_upload_with_3_retries_when_the_last_attempt_ends_then_one_upload_failure_per_n01_fr_10_and_tc_n_26()
 {
    let mut run = failing_with_3_retries_of_a_minute();

    for now in [10_000, 70_000, 130_000, 190_000] {
        tick(&mut run, now);
    }

    assert_eq!(
        notifications(&run),
        vec![
            json!({"status": "Uploading", "requestId": 4}),
            json!({"status": "UploadFailure", "requestId": 4})
        ]
    );
}

#[test]
fn given_booted_station_when_log_requested_then_accepted_with_its_file_name_and_upload_scheduled_per_n01_fr_01_and_tc_n_25()
 {
    let mut run = Run::accepted(86_400);

    let reply = get_log(&mut run, 3, "https://logs.example/");

    assert_eq!(
        reply,
        json!({"status": "Accepted", "filename": "CS1-security.log"})
    );
    assert_eq!(run.station.next_deadline(), Some(10_000));
}

#[test]
fn given_requested_log_when_upload_time_passes_then_uploading_then_uploaded_under_its_request_id_per_n01_fr_07_to_fr_09_and_tc_n_35()
 {
    let mut run = Run::accepted(86_400);
    get_log(&mut run, 3, "https://logs.example/");

    tick(&mut run, 10_000);

    assert_eq!(
        notifications(&run),
        vec![
            json!({"status": "Uploading", "requestId": 3}),
            json!({"status": "Uploaded", "requestId": 3})
        ]
    );
}

#[test]
fn given_booted_station_when_log_requested_to_an_unreachable_location_then_upload_failure_per_n01_fr_10()
 {
    let mut run = Run::accepted(86_400);

    get_log(&mut run, 3, "scp://logs.example/");

    assert_eq!(
        notifications(&run),
        vec![json!({"status": "UploadFailure", "requestId": 3})]
    );
}

#[test]
fn given_uploading_log_when_another_log_requested_then_first_canceled_and_second_uploading_per_n01_fr_12_fr_20_and_tc_n_36()
 {
    let mut run = Run::accepted(86_400);
    get_log(&mut run, 3, "https://logs.example/");

    let reply = get_log(&mut run, 4, "https://logs.example/");

    assert_eq!(reply["status"], "AcceptedCanceled");
    assert_eq!(
        notifications(&run)[1..],
        [
            json!({"status": "AcceptedCanceled", "requestId": 3}),
            json!({"status": "Uploading", "requestId": 4})
        ]
    );
}

#[test]
fn given_no_upload_when_log_status_triggered_then_idle_without_request_id_per_n01_fr_13_and_tc_f_15()
 {
    let mut run = Run::accepted(86_400);

    let status = trigger_status(&mut run);

    assert_eq!(status, "Accepted");
    assert_eq!(notifications(&run), vec![json!({"status": "Idle"})]);
}

#[test]
fn given_uploading_log_when_log_status_triggered_then_uploading_with_its_request_id_per_n01_fr_13_and_tc_f_16()
 {
    let mut run = Run::accepted(86_400);
    get_log(&mut run, 3, "https://logs.example/");

    trigger_status(&mut run);

    assert_eq!(
        notifications(&run).pop().unwrap(),
        json!({"status": "Uploading", "requestId": 3})
    );
}

#[test]
fn given_failing_upload_when_upload_time_passes_then_upload_failure_per_tc_n_26() {
    let mut run = Run::accepted(86_400);
    let stage = TransferStage::Upload;
    run.input(0, Input::Intent(Intent::FailTransfer { stage }));
    get_log(&mut run, 3, "https://logs.example/");

    tick(&mut run, 10_000);

    assert_eq!(
        notifications(&run),
        vec![
            json!({"status": "Uploading", "requestId": 3}),
            json!({"status": "UploadFailure", "requestId": 3})
        ]
    );
}
