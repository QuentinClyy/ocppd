use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::file_transfer::TransferStage;
use crate::fixtures::v201::firmware::*;
use crate::fixtures::v201::station::Run;
use crate::fixtures::x509::authority::Authority;
use serde_json::json;

#[test]
fn given_manufacturer_root_when_update_signed_by_a_stranger_requested_then_invalid_certificate_reported_per_l01_fr_22_and_fr_02_and_tc_l_05()
 {
    let mut run = Run::accepted(86_400);
    signer(&mut run);
    let stranger = Authority::new("stranger");
    let (_, csr) = crate::x509::signing::signing_request("forged signer", "").unwrap();
    let forged = String::from_utf8(stranger.sign(&csr).to_pem().unwrap()).unwrap();

    let status = update(&mut run, 0, secure(&forged));

    assert_eq!(status, "InvalidCertificate");
    assert_eq!(
        security_events(&run),
        vec!["InvalidFirmwareSigningCertificate"]
    );
    assert!(statuses(&run).is_empty());
}

#[test]
fn given_update_without_signature_when_downloaded_then_invalid_signature_reported_without_reboot_per_l01_fr_03()
 {
    let mut run = Run::accepted(86_400);
    let signer = signer(&mut run);
    let firmware =
        json!({"location": LOCATION, "retrieveDateTime": PAST, "signingCertificate": signer});
    update(&mut run, 0, firmware);

    tick(&mut run, 10_000);
    tick(&mut run, 30_000);

    assert_eq!(
        statuses(&run),
        vec!["Downloading", "Downloaded", "InvalidSignature"]
    );
    assert_eq!(security_events(&run), vec!["InvalidFirmwareSignature"]);
    assert!(!run.closed);
}

#[test]
fn given_booted_station_when_update_without_signing_certificate_requested_then_rejected_per_tc_l_18()
 {
    let mut run = Run::accepted(86_400);
    let firmware = json!({"location": LOCATION, "retrieveDateTime": PAST});

    let status = update(&mut run, 0, firmware);

    assert_eq!(status, "Rejected");
    assert!(statuses(&run).is_empty());
}

#[test]
fn given_failing_signature_check_when_downloaded_then_invalid_signature_reported_per_l01_fr_03_and_tc_l_06()
 {
    let mut run = failing(TransferStage::Signature);

    tick(&mut run, 10_000);

    assert_eq!(
        statuses(&run),
        vec!["Downloading", "Downloaded", "InvalidSignature"]
    );
    assert_eq!(security_events(&run), vec!["InvalidFirmwareSignature"]);
}

#[test]
fn given_trusted_signer_when_update_from_an_unreachable_location_requested_then_download_failed_per_l01_fr_30()
 {
    let mut run = Run::accepted(86_400);
    let mut firmware = secure(&signer(&mut run));
    firmware["location"] = json!("sftp://firmware.example/2.0.0");

    update(&mut run, 0, firmware);

    assert_eq!(statuses(&run), vec!["DownloadFailed"]);
    assert_eq!(run.station.next_deadline(), Some(86_400_000));
}

#[test]
fn given_failing_download_when_download_time_passes_then_download_failed_per_tc_l_07() {
    let mut run = failing(TransferStage::Download);

    tick(&mut run, 10_000);

    assert_eq!(statuses(&run), vec!["Downloading", "DownloadFailed"]);
    assert_eq!(run.station.next_deadline(), Some(86_400_000));
}

#[test]
fn given_failing_install_when_install_time_passes_then_installation_failed_without_reboot_per_tc_l_08()
 {
    let mut run = failing(TransferStage::Install);
    tick(&mut run, 10_000);

    tick(&mut run, 20_000);

    assert_eq!(statuses(&run).last(), Some(&json!("InstallationFailed")));
    assert!(!run.closed);
}

#[test]
fn given_trusted_signer_when_secure_update_requested_then_accepted_per_l01_fr_01() {
    let mut run = Run::accepted(86_400);
    let signer = signer(&mut run);

    let status = update(&mut run, 0, secure(&signer));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_accepted_update_when_download_time_passes_then_downloaded_verified_and_installing_per_l01_fr_04_and_tc_l_01()
 {
    let mut run = accepted_update();

    tick(&mut run, 10_000);

    assert_eq!(
        statuses(&run),
        vec![
            "Downloading",
            "Downloaded",
            "SignatureVerified",
            "Installing"
        ]
    );
}

#[test]
fn given_installing_firmware_when_install_time_passes_then_install_rebooting_per_l01_fr_05() {
    let mut run = downloaded();

    tick(&mut run, 20_000);

    assert_eq!(statuses(&run).last().unwrap(), "InstallRebooting");
    assert!(run.closed, "the station reboots to install");
}

#[test]
fn given_rebooting_to_install_when_rebooted_then_installed_with_the_new_version_under_one_request_id_per_l01_fr_10_fr_31_and_tc_l_01()
 {
    let mut run = rebooting_to_install();

    reboot(&mut run, 30_000);

    assert_eq!(statuses(&run).last().unwrap(), "Installed");
    let ids = sent(&run, "FirmwareStatusNotification");
    assert!(ids.iter().all(|payload| payload["requestId"] == 7));
    let boot = sent(&run, "BootNotification").pop().unwrap();
    assert_eq!(boot["reason"], "FirmwareUpdate");
    assert_eq!(boot["chargingStation"]["firmwareVersion"], "2.0.0");
    assert_eq!(security_events(&run).last().unwrap(), "FirmwareUpdated");
}

#[test]
fn given_trusted_signer_when_update_with_future_dates_requested_then_download_scheduled_per_l01_fr_13_and_tc_l_03()
 {
    let run = scheduled();

    let deadline = run.station.next_deadline();

    assert_eq!(statuses(&run), vec!["DownloadScheduled"]);
    assert_eq!(deadline, Some(60_000));
}

#[test]
fn given_scheduled_update_when_downloaded_before_the_install_date_then_install_scheduled_per_l01_fr_16_and_tc_l_02()
 {
    let mut run = scheduled();

    tick(&mut run, 60_000);
    tick(&mut run, 70_000);

    assert_eq!(statuses(&run).last().unwrap(), "InstallScheduled");
    assert_eq!(run.station.next_deadline(), Some(300_000));
}

#[test]
fn given_install_scheduled_when_the_install_date_comes_then_installing_per_l01_fr_16() {
    let mut run = scheduled();
    tick(&mut run, 60_000);
    tick(&mut run, 70_000);

    tick(&mut run, 300_000);

    assert_eq!(statuses(&run).last().unwrap(), "Installing");
}

#[test]
fn given_transaction_when_firmware_downloaded_then_waits_after_signature_verified_per_l01_fr_06() {
    let run = plugged_with_update();

    let last = statuses(&run).last().unwrap().clone();

    assert_eq!(last, "SignatureVerified");
}

#[test]
fn given_downloaded_firmware_waiting_for_a_transaction_when_it_ends_then_installing_per_l01_fr_06()
{
    let mut run = plugged_with_update();

    run.input(20_000, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(20_000);

    assert_eq!(statuses(&run).last().unwrap(), "Installing");
}

#[test]
fn given_pending_update_when_another_update_requested_then_accepted_canceled_per_l01_fr_24_and_tc_l_10()
 {
    let mut run = Run::accepted(86_400);
    let mut firmware = secure(&signer(&mut run));
    firmware["retrieveDateTime"] = json!("1970-01-01T01:00:00Z");
    update(&mut run, 0, firmware.clone());
    firmware["location"] = json!("https://firmware.example/3.0.0");

    let status = update(&mut run, 0, firmware);

    assert_eq!(status, "AcceptedCanceled");
}

#[test]
fn given_no_update_when_firmware_status_triggered_then_accepted_and_idle_per_l01_fr_25_and_tc_f_18()
{
    let mut run = Run::accepted(86_400);

    let status = trigger_status(&mut run);
    run.answer_all(0);

    assert_eq!(status, "Accepted");
    assert_eq!(
        sent(&run, "FirmwareStatusNotification").pop().unwrap(),
        json!({"status": "Idle"})
    );
}

#[test]
fn given_downloading_update_when_firmware_status_triggered_then_downloading_with_its_request_id_per_l01_fr_26_and_tc_f_19()
 {
    let mut run = accepted_update();

    let triggered = triggered(&mut run);

    assert_eq!(triggered, json!({"status": "Downloading", "requestId": 7}));
}

#[test]
fn given_installed_update_when_firmware_status_triggered_then_idle_per_l01_fr_25() {
    let mut run = rebooting_to_install();
    reboot(&mut run, 30_000);

    let triggered = triggered(&mut run);

    assert_eq!(triggered, json!({"status": "Idle"}));
}
