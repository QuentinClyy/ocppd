use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::protocol::shared::file_transfer::TransferStage;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

const RETRIEVE: &str = "1970-01-01T00:01:00Z";

fn update(csms: &mut Csms, now: u64, location: &str, retrieve: &str) {
    let request = json!({"location": location, "retrieveDate": retrieve});
    match csms.call(now, "UpdateFirmware", request) {
        Frame::CallResult { payload, .. } => assert_eq!(payload, json!({})),
        other => panic!("expected a result, got {other:?}"),
    }
}

fn statuses(csms: &Csms) -> Vec<Value> {
    let sent = csms.sent("FirmwareStatusNotification");
    sent.iter().map(|sent| sent["status"].clone()).collect()
}

fn updating(location: &str, retrieve: &str) -> Csms {
    let mut csms = Csms::booted();
    update(&mut csms, 0, location, retrieve);
    csms
}

fn failing(stage: TransferStage) -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, Intent::FailTransfer { stage });
    update(&mut csms, 0, "https://fw.example/v2", RETRIEVE);
    csms.tick(MINUTE);
    csms
}

const NEW_FIRMWARE: &str = "https://fw.example/ocppd-2.0.0.bin";

#[test]
fn given_booted_charge_point_when_update_requested_then_download_scheduled_at_the_retrieve_date() {
    let mut csms = Csms::booted();

    update(&mut csms, 0, NEW_FIRMWARE, RETRIEVE);

    assert_eq!(csms.charge_point.next_deadline(), Some(MINUTE));
}

#[test]
fn given_requested_update_when_the_retrieve_date_comes_then_downloading() {
    let mut csms = updating(NEW_FIRMWARE, RETRIEVE);

    csms.tick(MINUTE);

    assert_eq!(statuses(&csms), vec![json!("Downloading")]);
}

#[test]
fn given_downloading_firmware_when_the_download_ends_then_downloaded_and_installing_without_reboot_yet()
 {
    let mut csms = updating(NEW_FIRMWARE, RETRIEVE);
    csms.tick(MINUTE);

    csms.tick(MINUTE + 10_000);

    assert_eq!(
        statuses(&csms)[1..],
        [json!("Downloaded"), json!("Installing")]
    );
    assert!(!csms.closed);
}

#[test]
fn given_installing_firmware_when_the_install_ends_then_installed_reported_and_reboots() {
    let mut csms = updating(NEW_FIRMWARE, RETRIEVE);
    csms.tick(MINUTE);
    csms.tick(MINUTE + 10_000);

    csms.tick(MINUTE + 20_000);

    assert_eq!(statuses(&csms).last(), Some(&json!("Installed")));
    assert!(csms.closed, "the station reboots after installing");
    let reported = csms.sent("SecurityEventNotification");
    assert_eq!(reported[0]["type"], "FirmwareUpdated");
}

#[test]
fn given_installed_firmware_when_reconnected_then_boots_with_the_new_version() {
    let mut csms = updating(NEW_FIRMWARE, RETRIEVE);
    for now in [MINUTE, MINUTE + 10_000, MINUTE + 20_000] {
        csms.tick(now);
    }

    csms.reconnect(MINUTE + 30_000);

    let boot = csms.expect("BootNotification");
    assert_eq!(boot["firmwareVersion"], "ocppd-2.0.0.bin");
}

#[test]
fn given_transaction_when_firmware_downloaded_then_downloaded_but_not_installing() {
    let mut csms = Csms::charging(0, EvSpec::default());
    update(
        &mut csms,
        0,
        "https://fw.example/v2",
        "1970-01-01T00:00:00Z",
    );

    csms.tick(10_000);
    csms.tick(30_000);

    assert_eq!(
        statuses(&csms),
        vec![json!("Downloading"), json!("Downloaded")]
    );
}

#[test]
fn given_downloaded_firmware_waiting_for_a_transaction_when_it_stops_then_installing() {
    let mut csms = Csms::charging(0, EvSpec::default());
    update(
        &mut csms,
        0,
        "https://fw.example/v2",
        "1970-01-01T00:00:00Z",
    );
    csms.tick(10_000);

    csms.intent(40_000, Intent::Stop { connector: 1 });

    assert_eq!(statuses(&csms).last(), Some(&json!("Installing")));
}

#[test]
fn given_update_from_a_file_location_when_the_retrieve_date_comes_then_download_failed() {
    let mut csms = updating("file:///tmp/fw.bin", RETRIEVE);

    csms.tick(MINUTE);

    assert_eq!(statuses(&csms), vec![json!("DownloadFailed")]);
    assert_eq!(csms.charge_point.next_deadline(), Some(86_400_000));
}

#[test]
fn given_location_without_a_file_name_when_installed_then_installation_failed_without_reboot() {
    let mut csms = updating("https://fw.example/", "1970-01-01T00:00:00Z");

    csms.tick(10_000);
    csms.tick(20_000);

    assert_eq!(statuses(&csms).last(), Some(&json!("InstallationFailed")));
    assert!(!csms.closed);
}

#[test]
fn given_downloading_firmware_when_its_status_triggered_then_downloading_reported_again() {
    let mut csms = updating("https://fw.example/v2", RETRIEVE);
    csms.tick(MINUTE);
    let request = json!({"requestedMessage": "FirmwareStatusNotification"});

    csms.call(MINUTE + 1_000, "TriggerMessage", request);

    assert_eq!(
        statuses(&csms),
        vec![json!("Downloading"), json!("Downloading")]
    );
}

#[test]
fn given_booted_charge_point_when_update_location_is_not_a_uri_then_property_constraint_violation()
{
    let mut csms = Csms::booted();
    let request = json!({"location": "ocppd-2.0.0.bin", "retrieveDate": RETRIEVE});

    let reply = csms.call(0, "UpdateFirmware", request);

    match reply {
        Frame::CallError { code, .. } => assert_eq!(code, "PropertyConstraintViolation"),
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn given_failing_download_when_the_download_ends_then_download_failed_per_tc_044_2() {
    let mut csms = failing(TransferStage::Download);

    csms.tick(MINUTE + 10_000);

    assert_eq!(
        statuses(&csms),
        vec![json!("Downloading"), json!("DownloadFailed")]
    );
    assert_eq!(csms.charge_point.next_deadline(), Some(86_400_000));
}

#[test]
fn given_failing_install_when_the_install_ends_then_installation_failed_without_reboot_per_tc_044_3()
 {
    let mut csms = failing(TransferStage::Install);
    csms.tick(MINUTE + 10_000);

    csms.tick(MINUTE + 20_000);

    assert_eq!(statuses(&csms).last(), Some(&json!("InstallationFailed")));
    assert!(!csms.closed);
}
