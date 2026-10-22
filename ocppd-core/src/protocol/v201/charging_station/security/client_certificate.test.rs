use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use crate::fixtures::x509::authority::{Authority, new_key};
use crate::protocol::v201::event::EventKind;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::x509::{X509, X509NameBuilder, X509Req, X509ReqBuilder};
use serde_json::{Value, json};

const RENEW_BEFORE_MS: u64 = 30 * 86_400_000;
const VALID_UNTIL_MS: u64 = 4_000_000_000_000;

fn sent(run: &Run, action: &str) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, sent, _)| sent == action)
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

fn trigger(run: &mut Run) -> String {
    let request = json!({"requestedMessage": "SignChargingStationCertificate"});
    assert_eq!(
        run.result(0, "TriggerMessage", request)["status"],
        "Accepted"
    );
    let csr = run.last_call().2["csr"].as_str().unwrap().to_owned();
    run.answer(0, json!({"status": "Accepted"}));
    run.answer_all(0);
    csr
}

fn signed(run: &mut Run, now: u64, chain: &[&X509]) -> Value {
    let pem: Vec<u8> = chain
        .iter()
        .flat_map(|cert| cert.to_pem().unwrap())
        .collect();
    let request = json!({"certificateChain": String::from_utf8(pem).unwrap(), "certificateType": "ChargingStationCertificate"});
    let status = run.result(now, "CertificateSigned", request)["status"].clone();
    run.answer_all(now);
    status
}

fn security_events(run: &Run) -> Vec<Value> {
    let sent = sent(run, "SecurityEventNotification");
    sent.iter().map(|payload| payload["type"].clone()).collect()
}

fn subject(csr: &str, nid: Nid) -> String {
    let request = X509Req::from_pem(csr.as_bytes()).unwrap();
    let entry = request.subject_name().entries_by_nid(nid).next().unwrap();
    entry.data().to_string().unwrap()
}

fn renamed(csr: &str, common_name: &str) -> String {
    let request = X509Req::from_pem(csr.as_bytes()).unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_nid(Nid::COMMONNAME, common_name)
        .unwrap();
    let mut builder = X509ReqBuilder::new().unwrap();
    builder.set_subject_name(&name.build()).unwrap();
    builder.set_pubkey(&request.public_key().unwrap()).unwrap();
    builder.sign(&new_key(), MessageDigest::sha256()).unwrap();
    String::from_utf8(builder.build().to_pem().unwrap()).unwrap()
}

fn foreign_leaf() -> X509 {
    let (_, other_csr) = crate::x509::signing::signing_request("SN-1", "").unwrap();
    Authority::new("CSO CA").sign(&other_csr)
}

fn installed() -> Run {
    let mut run = Run::accepted(86_400);
    let csr = trigger(&mut run);
    let authority = Authority::new("CSO CA");
    signed(
        &mut run,
        0,
        &[&authority.sign(&csr), &authority.certificate],
    );
    run
}

fn one_repeat() -> (Run, String) {
    let mut run = Run::accepted(86_400);
    assert!(
        run.station
            .configure("SecurityCtrlr.CertSigningRepeatTimes", "1")
    );
    let csr = trigger(&mut run);
    (run, csr)
}

fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

#[test]
fn given_organization_name_when_certificate_triggered_then_csr_for_the_serial_and_organization_per_a02_fr_03_and_a00_fr_509()
 {
    let mut run = Run::accepted(86_400);
    assert!(
        run.station
            .configure("SecurityCtrlr.OrganizationName", "ocppd CSO")
    );

    let csr = trigger(&mut run);

    assert_eq!(subject(&csr, Nid::COMMONNAME), "SN-1", "A00.FR.511");
    assert_eq!(subject(&csr, Nid::ORGANIZATIONNAME), "ocppd CSO");
    let request = sent(&run, "SignCertificate").pop().unwrap();
    assert_eq!(request["certificateType"], "ChargingStationCertificate");
}

#[test]
fn given_sent_csr_when_valid_chain_signed_then_accepted_and_reconnects_to_switch_per_a02_fr_08() {
    let mut run = Run::accepted(86_400);
    let csr = trigger(&mut run);
    let authority = Authority::new("CSO CA");
    let leaf = authority.sign(&csr);

    let status = signed(&mut run, 0, &[&leaf, &authority.certificate]);

    assert_eq!(status, "Accepted");
    assert!(
        run.events
            .contains(&EventKind::ChargingStationCertificateInstalled)
    );
    assert!(run.closed, "the station reconnects to switch certificates");
}

#[test]
fn given_installed_certificate_on_profile_2_when_connecting_then_not_presented_per_a02_fr_08() {
    let run = installed();

    let client = run.station.client_certificate();

    assert!(client.is_none(), "unused below profile 3");
}

#[test]
fn given_installed_certificate_on_profile_3_when_connecting_then_its_chain_presented_per_a02_fr_08()
{
    let mut run = installed();
    assert!(run.station.configure("SecurityCtrlr.SecurityProfile", "3"));

    let client = run.station.client_certificate().unwrap();

    assert_eq!(X509::stack_from_pem(&client.chain_pem).unwrap().len(), 2);
}

#[test]
fn given_sent_csr_when_certificate_for_another_key_signed_then_rejected_and_reported_per_a02_fr_07()
{
    let mut run = Run::accepted(86_400);
    trigger(&mut run);

    let status = signed(&mut run, 0, &[&foreign_leaf()]);

    assert_eq!(status, "Rejected");
    assert_eq!(
        security_events(&run),
        vec!["InvalidChargingStationCertificate"]
    );
    assert!(!run.closed);
}

#[test]
fn given_sent_csr_when_certificate_for_another_station_signed_then_rejected_per_a02_fr_06() {
    let mut run = Run::accepted(86_400);
    let csr = trigger(&mut run);
    let leaf = Authority::new("CSO CA").sign(&renamed(&csr, "SN-2"));

    let status = signed(&mut run, 0, &[&leaf]);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_accepted_csr_when_just_before_the_wait_then_not_resent_per_a02_fr_17() {
    let (mut run, _) = one_repeat();

    run.input(29_999, Input::Tick);

    assert_eq!(sent(&run, "SignCertificate").len(), 1);
}

#[test]
fn given_accepted_csr_when_the_wait_expires_then_same_csr_resent_per_a02_fr_17_and_tc_a_23() {
    let (mut run, csr) = one_repeat();

    run.input(30_000, Input::Tick);

    assert_eq!(sent(&run, "SignCertificate").len(), 2);
    assert_eq!(run.last_call().2["csr"], csr.as_str(), "the same CSR");
}

#[test]
fn given_one_resend_when_just_before_double_the_wait_then_not_resent_per_a02_fr_18() {
    let (mut run, _) = one_repeat();
    run.input(30_000, Input::Tick);
    run.answer(30_000, json!({"status": "Accepted"}));

    run.input(89_999, Input::Tick);

    assert_eq!(sent(&run, "SignCertificate").len(), 2);
}

#[test]
fn given_one_resend_when_double_the_wait_expires_then_resent_again_per_a02_fr_18_and_tc_a_23() {
    let (mut run, csr) = one_repeat();
    run.input(30_000, Input::Tick);
    run.answer(30_000, json!({"status": "Accepted"}));

    run.input(90_000, Input::Tick);

    assert_eq!(sent(&run, "SignCertificate").len(), 3);
    assert_eq!(run.last_call().2["csr"], csr.as_str(), "the same CSR");
}

#[test]
fn given_repeats_used_up_when_time_passes_then_no_more_resends_per_a02_fr_19_and_tc_a_23() {
    let (mut run, _) = one_repeat();
    for at in [30_000, 90_000] {
        run.input(at, Input::Tick);
        run.answer(at, json!({"status": "Accepted"}));
    }

    tick(&mut run, 3_600_000);

    assert_eq!(
        sent(&run, "SignCertificate").len(),
        3,
        "one doubling, then it stops"
    );
}

#[test]
fn given_rejected_certificate_when_the_wait_expires_then_csr_not_resent_per_a02_fr_20() {
    let mut run = Run::accepted(86_400);
    trigger(&mut run);
    signed(&mut run, 0, &[&foreign_leaf()]);

    tick(&mut run, 30_000);

    assert_eq!(sent(&run, "SignCertificate").len(), 1);
}

#[test]
fn given_installed_certificate_when_just_before_its_renewal_time_then_no_csr_per_a03_fr_02() {
    let mut run = installed();

    tick(&mut run, VALID_UNTIL_MS - RENEW_BEFORE_MS - 1);

    assert_eq!(sent(&run, "SignCertificate").len(), 1);
}

#[test]
fn given_installed_certificate_when_its_renewal_time_comes_then_new_csr_sent_per_a03_fr_02() {
    let mut run = installed();

    tick(&mut run, VALID_UNTIL_MS - RENEW_BEFORE_MS);

    assert_eq!(sent(&run, "SignCertificate").len(), 2);
}
