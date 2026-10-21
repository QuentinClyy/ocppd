use crate::api::event::EventKind;
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::{result, security_events};
use crate::fixtures::x509::authority::{Authority, new_key};
use openssl::nid::Nid;
use openssl::x509::{X509, X509Req};
use serde_json::{Value, json};

const NOW: u64 = 1_798_761_600_000;

type Case = fn(&mut Csms, &Authority, &Authority) -> Value;

fn request_csr(csms: &mut Csms) -> String {
    let trigger = json!({"requestedMessage": "SignChargePointCertificate"});
    let status = result(csms.call(NOW, "ExtendedTriggerMessage", trigger))["status"].clone();
    assert_eq!(status, "Accepted");
    let csr = csms.expect("SignCertificate")["csr"]
        .as_str()
        .unwrap()
        .to_owned();
    csms.answer(NOW, json!({"status": "Accepted"}));
    csr
}

fn signed(csms: &mut Csms, chain: &[&X509]) -> Value {
    let pem: Vec<u8> = chain
        .iter()
        .flat_map(|cert| cert.to_pem().unwrap())
        .collect();
    let request = json!({"certificateChain": String::from_utf8(pem).unwrap()});
    result(csms.call(NOW, "CertificateSigned", request))["status"].clone()
}

fn rejected_csr() -> (Csms, String) {
    let mut csms = Csms::booted();
    let trigger = json!({"requestedMessage": "SignChargePointCertificate"});
    csms.call(NOW, "ExtendedTriggerMessage", trigger);
    let csr = csms.expect("SignCertificate")["csr"]
        .as_str()
        .unwrap()
        .to_owned();
    csms.answer(NOW, json!({"status": "Rejected"}));
    (csms, csr)
}

fn installed_chain() -> (Csms, Authority, X509) {
    let mut csms = Csms::booted();
    let authority = Authority::new("CSMS CA");
    let leaf = authority.sign(&request_csr(&mut csms));
    signed(&mut csms, &[&leaf, &authority.certificate]);
    (csms, authority, leaf)
}

fn entry(name: &openssl::x509::X509NameRef, nid: Nid) -> Option<String> {
    let entry = name.entries_by_nid(nid).next()?;
    entry.data().to_string().ok()
}

#[test]
fn given_cpo_name_when_certificate_triggered_then_csr_names_the_serial_number_and_the_cpo() {
    let mut csms = Csms::booted();
    csms.configure(NOW, "CpoName", "Example CPO");

    let csr = X509Req::from_pem(request_csr(&mut csms).as_bytes()).unwrap();

    let subject = csr.subject_name();
    assert_eq!(entry(subject, Nid::COMMONNAME).as_deref(), Some("CP1"));
    assert_eq!(
        entry(subject, Nid::ORGANIZATIONNAME).as_deref(),
        Some("Example CPO")
    );
    let key = csr.public_key().unwrap();
    assert!(csr.verify(&key).unwrap(), "signed with its own key");
}

#[test]
fn given_sent_csr_when_chain_for_its_key_signed_then_accepted_and_reported_but_unused_at_profile_0()
{
    let mut csms = Csms::booted();
    let authority = Authority::new("CSMS CA");
    let leaf = authority.sign(&request_csr(&mut csms));
    csms.events.clear();

    let status = signed(&mut csms, &[&leaf, &authority.certificate]);

    assert_eq!(status, "Accepted");
    assert_eq!(
        csms.events,
        vec![EventKind::ChargePointCertificateInstalled]
    );
    assert!(
        csms.charge_point.client_certificate().is_none(),
        "profile 0"
    );
}

#[test]
fn given_installed_chain_and_csms_root_when_profile_3_set_then_accepted_and_chain_presented_with_its_key()
 {
    let (mut csms, authority, leaf) = installed_chain();
    let root =
        json!({"certificateType": "CentralSystemRootCertificate", "certificate": authority.pem()});
    result(csms.call(NOW, "InstallCertificate", root));

    let status = csms.configure(NOW, "SecurityProfile", "3");

    assert_eq!(status, "Accepted");
    let client = csms.charge_point.client_certificate().unwrap();
    let chain = X509::stack_from_pem(&client.chain_pem).unwrap();
    assert_eq!(chain, vec![leaf.clone(), authority.certificate.clone()]);
    let key = openssl::pkey::PKey::private_key_from_pem(&client.key_pkcs8_pem).unwrap();
    assert!(leaf.public_key().unwrap().public_eq(&key));
}

#[test]
fn given_each_unfitting_chain_when_signed_then_rejected_and_invalid_certificate_reported() {
    let authority = Authority::new("CSMS CA");
    let stranger = Authority::new("stranger");
    let cases: [(&str, Case); 4] = [
        ("garbage", |csms, _, _| {
            request_csr(csms);
            let request = json!({"certificateChain": "garbage"});
            result(csms.call(NOW, "CertificateSigned", request))["status"].clone()
        }),
        ("another key", |csms, authority, _| {
            request_csr(csms);
            let mut other = X509Req::builder().unwrap();
            let key = new_key();
            other.set_pubkey(&key).unwrap();
            other
                .sign(&key, openssl::hash::MessageDigest::sha256())
                .unwrap();
            let pem = String::from_utf8(other.build().to_pem().unwrap()).unwrap();
            signed(csms, &[&authority.sign(&pem)])
        }),
        ("broken chain", |csms, authority, stranger| {
            let leaf = authority.sign(&request_csr(csms));
            signed(csms, &[&leaf, &stranger.certificate])
        }),
        ("no request", |csms, _, stranger| {
            signed(csms, &[&stranger.certificate])
        }),
    ];
    for (case, run) in cases {
        let mut csms = Csms::booted();

        let status = run(&mut csms, &authority, &stranger);

        assert_eq!(status, "Rejected", "{case}");
        let events = security_events(&csms);
        assert_eq!(
            events,
            vec![json!("InvalidChargePointCertificate")],
            "{case}"
        );
    }
}

#[test]
fn given_rejected_csr_when_certificate_for_it_signed_then_rejected() {
    let (mut csms, csr) = rejected_csr();
    let authority = Authority::new("CSMS CA");

    let status = signed(&mut csms, &[&authority.sign(&csr)]);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_sent_csr_when_leaf_signed_then_accepted() {
    let mut csms = Csms::booted();
    let leaf = Authority::new("CSMS CA").sign(&request_csr(&mut csms));

    let status = signed(&mut csms, &[&leaf]);

    assert_eq!(status, "Accepted");
}

#[test]
fn given_installed_certificate_when_signed_again_then_rejected() {
    let (mut csms, _, leaf) = installed_chain();

    let status = signed(&mut csms, &[&leaf]);

    assert_eq!(status, "Rejected", "no request pending");
}
