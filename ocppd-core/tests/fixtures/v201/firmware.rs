use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::file_transfer::TransferStage;
use crate::fixtures::v201::station::{Run, boot};
use crate::fixtures::x509::authority::Authority;
use serde_json::{Value, json};

pub const LOCATION: &str = "https://firmware.example/2.0.0";
pub const PAST: &str = "1970-01-01T00:00:00Z";

pub fn sent(run: &Run, action: &str) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, sent, _)| sent == action)
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn statuses(run: &Run) -> Vec<Value> {
    let sent = sent(run, "FirmwareStatusNotification");
    sent.iter()
        .map(|payload| payload["status"].clone())
        .collect()
}

pub fn security_events(run: &Run) -> Vec<Value> {
    let sent = sent(run, "SecurityEventNotification");
    sent.iter().map(|payload| payload["type"].clone()).collect()
}

pub fn signer(run: &mut Run) -> String {
    let maker = Authority::new("maker root");
    let request =
        json!({"certificateType": "ManufacturerRootCertificate", "certificate": maker.pem()});
    assert_eq!(
        run.result(0, "InstallCertificate", request)["status"],
        "Accepted"
    );
    let (_, csr) = crate::x509::signing::signing_request("firmware signer", "").unwrap();
    String::from_utf8(maker.sign(&csr).to_pem().unwrap()).unwrap()
}

pub fn update(run: &mut Run, now: u64, firmware: Value) -> Value {
    let request = json!({"requestId": 7, "firmware": firmware});
    let status = run.result(now, "UpdateFirmware", request)["status"].clone();
    run.answer_all(now);
    status
}

pub fn secure(signer: &str) -> Value {
    json!({"location": LOCATION, "retrieveDateTime": PAST, "signingCertificate": signer, "signature": "c2lnbmF0dXJl"})
}

pub fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

pub fn reboot(run: &mut Run, now: u64) {
    run.reconnect(now);
    run.answer(now, boot("Accepted", 86_400));
    run.answer_all(now);
}

pub fn accepted_update() -> Run {
    let mut run = Run::accepted(86_400);
    let signer = signer(&mut run);
    update(&mut run, 0, secure(&signer));
    run
}

pub fn downloaded() -> Run {
    let mut run = accepted_update();
    tick(&mut run, 10_000);
    run
}

pub fn rebooting_to_install() -> Run {
    let mut run = downloaded();
    tick(&mut run, 20_000);
    run
}

pub fn scheduled() -> Run {
    let mut run = Run::accepted(86_400);
    let signer = signer(&mut run);
    let mut firmware = secure(&signer);
    firmware["retrieveDateTime"] = json!("1970-01-01T00:01:00Z");
    firmware["installDateTime"] = json!("1970-01-01T00:05:00Z");
    update(&mut run, 0, firmware);
    run
}

pub fn plugged_with_update() -> Run {
    let mut run = Run::accepted(86_400);
    let signer = signer(&mut run);
    let ev = EvSpec::default();
    run.input(0, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(0);
    update(&mut run, 0, secure(&signer));
    tick(&mut run, 10_000);
    run
}

pub fn trigger_status(run: &mut Run) -> Value {
    let trigger = json!({"requestedMessage": "FirmwareStatusNotification"});
    run.result(0, "TriggerMessage", trigger)["status"].clone()
}

pub fn triggered(run: &mut Run) -> Value {
    trigger_status(run);
    run.answer_all(0);
    sent(run, "FirmwareStatusNotification").pop().unwrap()
}

pub fn failing(stage: TransferStage) -> Run {
    let mut run = Run::accepted(86_400);
    run.input(0, Input::Intent(Intent::FailTransfer { stage }));
    let firmware = secure(&signer(&mut run));
    update(&mut run, 0, firmware);
    run
}
