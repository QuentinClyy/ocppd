use crate::api::intent::Intent;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::protocol::shared::file_transfer::TransferStage;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn signed_update(csms: &mut Csms, now: u64, firmware: Value) -> Value {
    let mut base = json!({
        "location": "https://fw.example/ocppd-3.0.0.bin",
        "retrieveDateTime": "1970-01-01T00:01:00Z",
        "signingCertificate": "-----BEGIN CERTIFICATE-----",
        "signature": "c2lnbmF0dXJl",
    });
    if let (Some(base), Some(extra)) = (base.as_object_mut(), firmware.as_object()) {
        base.extend(extra.clone());
    }
    let request = json!({"requestId": 3, "firmware": base});
    match csms.call(now, "SignedUpdateFirmware", request) {
        Frame::CallResult { payload, .. } => payload["status"].clone(),
        other => panic!("expected a result, got {other:?}"),
    }
}

fn signed_statuses(csms: &Csms) -> Vec<Value> {
    let sent = csms.sent("SignedFirmwareStatusNotification");
    sent.iter().map(|sent| sent["status"].clone()).collect()
}

fn security_events(csms: &Csms) -> Vec<Value> {
    let sent = csms.sent("SecurityEventNotification");
    sent.iter().map(|sent| sent["type"].clone()).collect()
}

fn downloaded(firmware: Value) -> Csms {
    let mut csms = Csms::booted();
    signed_update(&mut csms, 0, firmware);
    csms.tick(MINUTE);
    csms.tick(MINUTE + 10_000);
    csms
}

fn rebooting() -> Csms {
    let mut csms = downloaded(json!({}));
    csms.tick(MINUTE + 20_000);
    csms
}

fn rebooted() -> Csms {
    let mut csms = rebooting();
    csms.disconnect(MINUTE + 20_000);
    csms.reconnect(MINUTE + 30_000);
    csms
}

#[test]
fn given_booted_charge_point_when_signed_update_requested_then_accepted() {
    let mut csms = Csms::booted();

    let status = signed_update(&mut csms, 0, json!({}));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_signed_update_when_downloaded_and_installed_then_every_step_reported_and_reboots() {
    let mut csms = downloaded(json!({}));

    csms.tick(MINUTE + 20_000);

    assert!(csms.closed);
    assert_eq!(
        signed_statuses(&csms),
        vec![
            json!("DownloadScheduled"),
            json!("Downloading"),
            json!("Downloaded"),
            json!("SignatureVerified"),
            json!("Installing"),
            json!("InstallRebooting"),
        ]
    );
}

#[test]
fn given_rebooting_to_install_when_reconnected_then_boots_with_the_new_version() {
    let mut csms = rebooting();
    csms.disconnect(MINUTE + 20_000);

    csms.reconnect(MINUTE + 30_000);

    let boot = csms.expect("BootNotification");
    assert_eq!(boot["firmwareVersion"], "ocppd-3.0.0.bin");
}

#[test]
fn given_rebooted_with_new_firmware_when_boot_accepted_then_installed_and_firmware_updated_reported_only_as_signed()
 {
    let mut csms = rebooted();

    csms.answer(
        MINUTE + 30_000,
        json!({"status": "Accepted", "currentTime": "2026-10-20T00:00:00Z", "interval": 300}),
    );

    let installed = csms.sent("SignedFirmwareStatusNotification").pop().unwrap();
    assert_eq!(installed, json!({"status": "Installed", "requestId": 3}));
    assert!(security_events(&csms).contains(&json!("FirmwareUpdated")));
    assert!(csms.sent("FirmwareStatusNotification").is_empty());
}

#[test]
fn given_signed_update_with_install_date_when_downloaded_then_install_scheduled_until_then() {
    let csms = downloaded(json!({"installDateTime": "1970-01-01T00:05:00Z"}));

    let last = signed_statuses(&csms).last().cloned();

    assert_eq!(last, Some(json!("InstallScheduled")));
    assert_eq!(csms.charge_point.next_deadline(), Some(5 * MINUTE));
}

#[test]
fn given_install_scheduled_when_the_install_date_comes_then_installing() {
    let mut csms = downloaded(json!({"installDateTime": "1970-01-01T00:05:00Z"}));

    csms.tick(5 * MINUTE);

    assert_eq!(signed_statuses(&csms).last(), Some(&json!("Installing")));
}

#[test]
fn given_signed_update_without_signature_when_downloaded_then_invalid_signature_reported_without_reboot()
 {
    let mut csms = downloaded(json!({"signature": ""}));

    csms.tick(MINUTE + 20_000);

    assert_eq!(
        signed_statuses(&csms).last(),
        Some(&json!("InvalidSignature"))
    );
    assert_eq!(
        security_events(&csms),
        vec![json!("InvalidFirmwareSignature")]
    );
    assert!(!csms.closed);
}

#[test]
fn given_failing_signature_check_when_downloaded_then_invalid_signature_reported() {
    let mut csms = Csms::booted();
    let stage = TransferStage::Signature;
    csms.intent(0, Intent::FailTransfer { stage });
    signed_update(&mut csms, 0, json!({}));
    csms.tick(MINUTE);

    csms.tick(MINUTE + 10_000);

    assert_eq!(
        signed_statuses(&csms).last(),
        Some(&json!("InvalidSignature"))
    );
    assert_eq!(
        security_events(&csms),
        vec![json!("InvalidFirmwareSignature")]
    );
}

#[test]
fn given_booted_charge_point_when_signed_update_without_signing_certificate_then_invalid_certificate_reported()
 {
    let mut csms = Csms::booted();

    let status = signed_update(&mut csms, 0, json!({"signingCertificate": ""}));

    assert_eq!(status, "InvalidCertificate");
    assert_eq!(
        security_events(&csms),
        vec![json!("InvalidFirmwareSigningCertificate")]
    );
}

#[test]
fn given_update_in_progress_when_another_signed_update_requested_then_accepted_canceled() {
    let mut csms = Csms::booted();
    signed_update(&mut csms, 0, json!({}));

    let status = signed_update(&mut csms, 0, json!({}));

    assert_eq!(status, "AcceptedCanceled");
}

#[test]
fn given_no_update_when_extended_trigger_asks_firmware_status_then_signed_idle() {
    let mut csms = Csms::booted();
    let trigger = json!({"requestedMessage": "FirmwareStatusNotification"});

    csms.call(0, "ExtendedTriggerMessage", trigger);

    let sent = csms.sent("SignedFirmwareStatusNotification");
    assert_eq!(sent, vec![json!({"status": "Idle"})]);
}

#[test]
fn given_downloading_signed_update_when_extended_trigger_asks_firmware_status_then_signed_downloading_with_its_request_id()
 {
    let mut csms = Csms::booted();
    signed_update(&mut csms, 0, json!({}));
    csms.tick(MINUTE);
    let trigger = json!({"requestedMessage": "FirmwareStatusNotification"});

    csms.call(MINUTE, "ExtendedTriggerMessage", trigger);

    let sent = csms.sent("SignedFirmwareStatusNotification");
    assert_eq!(
        sent.last(),
        Some(&json!({"status": "Downloading", "requestId": 3}))
    );
}
