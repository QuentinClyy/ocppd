use crate::fixtures::v201::certificates::{CSMS_ROOT, entries, install, signed_by};
use crate::fixtures::v201::station::Run;
use crate::fixtures::x509::authority::Authority;
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

fn listed(run: &mut Run, kinds: Value) -> Value {
    run.result(
        0,
        "GetInstalledCertificateIds",
        json!({"certificateType": kinds}),
    )
}

fn delete(run: &mut Run, hash: &Value) -> Value {
    let request = json!({"certificateHashData": hash});
    run.result(0, "DeleteCertificate", request)["status"].clone()
}

fn first_hash(run: &mut Run, kind: &str) -> Value {
    let reply = listed(run, json!([kind]));
    reply["certificateHashDataChain"][0]["certificateHashData"].clone()
}

fn roots(count: usize) -> Vec<Authority> {
    (0..count)
        .map(|n| Authority::new(&format!("root {n}")))
        .collect()
}

#[test]
fn given_booted_station_when_csms_root_installed_then_accepted_listed_counted_and_trusted_per_m05_fr_02_and_m03_fr_04()
 {
    let mut run = Run::accepted(86_400);
    let root = Authority::new("CSMS root");

    let status = install(&mut run, CSMS_ROOT, &root.pem());

    assert_eq!(status, "Accepted");
    assert_eq!(
        run.events,
        vec![EventKind::CertificateInstalled {
            certificate_type: serde_json::from_value(json!(CSMS_ROOT)).unwrap()
        }]
    );
    let reply = listed(&mut run, json!([CSMS_ROOT]));
    assert_eq!(reply["status"], "Accepted");
    let chain = reply["certificateHashDataChain"].as_array().unwrap();
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0]["certificateType"], CSMS_ROOT);
    assert_eq!(chain[0]["certificateHashData"]["serialNumber"], "1");
    assert_eq!(entries(&mut run), "1");
    assert_eq!(
        run.station.csms_roots(),
        vec![root.certificate.to_pem().unwrap()]
    );
}

#[test]
fn given_booted_station_when_garbage_installed_then_rejected_per_m05_fr_07() {
    let mut run = Run::accepted(86_400);

    let status = install(&mut run, CSMS_ROOT, "not a certificate");

    assert_eq!(status, "Rejected");
    assert_eq!(entries(&mut run), "0");
}

#[test]
fn given_booted_station_when_leaf_installed_as_root_then_rejected_as_not_self_signed_per_m05_fr_07()
{
    let mut run = Run::accepted(86_400);
    let leaf = signed_by(&Authority::new("CSMS root"), "leaf");

    let status = install(&mut run, CSMS_ROOT, &leaf);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_station_when_expired_root_installed_then_rejected_per_m05_fr_07_and_tc_m_07() {
    let mut run = Run::accepted(86_400);
    let expired =
        json!({"certificateType": CSMS_ROOT, "certificate": Authority::new("CSMS root").pem()});

    let status = run.result(4_000_000_001_000, "InstallCertificate", expired)["status"].clone();

    assert_eq!(status, "Rejected");
    assert_eq!(entries(&mut run), "0");
}

#[test]
fn given_full_store_when_new_certificate_installed_then_rejected_per_m05_fr_06() {
    let mut run = Run::accepted(86_400);
    let roots = roots(11);
    for root in &roots[..10] {
        install(&mut run, "MORootCertificate", &root.pem());
    }

    let status = install(&mut run, "MORootCertificate", &roots[10].pem());

    assert_eq!(status, "Rejected");
    assert_eq!(entries(&mut run), "10");
}

#[test]
fn given_full_store_when_present_certificate_installed_again_then_replaced_per_m05_fr_17() {
    let mut run = Run::accepted(86_400);
    let roots = roots(10);
    for root in &roots {
        install(&mut run, "MORootCertificate", &root.pem());
    }

    let status = install(&mut run, "MORootCertificate", &roots[0].pem());

    assert_eq!(status, "Accepted");
    assert_eq!(entries(&mut run), "10");
}

#[test]
fn given_v2g_and_csms_roots_when_mo_roots_listed_then_not_found_per_m03_fr_02_and_tc_m_19() {
    let mut run = Run::accepted(86_400);
    install(&mut run, "V2GRootCertificate", &Authority::new("v2g").pem());
    install(&mut run, CSMS_ROOT, &Authority::new("csms").pem());

    let reply = listed(&mut run, json!(["MORootCertificate"]));

    assert_eq!(reply["status"], "NotFound");
}

#[test]
fn given_v2g_and_csms_roots_when_v2g_roots_listed_then_only_the_v2g_root_per_m03_fr_02() {
    let mut run = Run::accepted(86_400);
    install(&mut run, "V2GRootCertificate", &Authority::new("v2g").pem());
    install(&mut run, CSMS_ROOT, &Authority::new("csms").pem());

    let reply = listed(&mut run, json!(["V2GRootCertificate"]));

    assert_eq!(
        reply["certificateHashDataChain"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn given_v2g_and_csms_roots_when_listed_without_type_then_both_per_m03_fr_04_and_tc_m_18() {
    let mut run = Run::accepted(86_400);
    install(&mut run, "V2GRootCertificate", &Authority::new("v2g").pem());
    install(&mut run, CSMS_ROOT, &Authority::new("csms").pem());

    let all = run.result(0, "GetInstalledCertificateIds", json!({}));

    assert_eq!(all["certificateHashDataChain"].as_array().unwrap().len(), 2);
}

#[test]
fn given_listed_mo_root_when_deleted_by_its_hash_then_accepted_and_gone_per_m04_fr_02() {
    let mut run = Run::accepted(86_400);
    install(&mut run, "MORootCertificate", &Authority::new("mo").pem());
    let hash = first_hash(&mut run, "MORootCertificate");

    let status = delete(&mut run, &hash);

    assert_eq!(status, "Accepted");
    assert!(
        run.events
            .iter()
            .any(|kind| matches!(kind, EventKind::CertificateDeleted { .. }))
    );
    assert_eq!(entries(&mut run), "0");
}

#[test]
fn given_deleted_certificate_when_deleted_again_then_not_found_per_m04_fr_04_and_tc_m_22() {
    let mut run = Run::accepted(86_400);
    install(&mut run, "MORootCertificate", &Authority::new("mo").pem());
    let hash = first_hash(&mut run, "MORootCertificate");
    delete(&mut run, &hash);

    let status = delete(&mut run, &hash);

    assert_eq!(status, "NotFound");
}

#[test]
fn given_last_csms_root_on_profile_2_when_deleted_then_failed_per_m04_fr_03() {
    let mut run = Run::accepted(86_400);
    install(&mut run, CSMS_ROOT, &Authority::new("csms").pem());
    let hash = first_hash(&mut run, CSMS_ROOT);
    assert!(run.station.configure("SecurityCtrlr.SecurityProfile", "2"));

    let status = delete(&mut run, &hash);

    assert_eq!(status, "Failed");
}

#[test]
fn given_last_csms_root_on_profile_1_when_deleted_then_accepted_per_m04_fr_03() {
    let mut run = Run::accepted(86_400);
    install(&mut run, CSMS_ROOT, &Authority::new("csms").pem());
    let hash = first_hash(&mut run, CSMS_ROOT);
    assert!(run.station.configure("SecurityCtrlr.SecurityProfile", "1"));

    let status = delete(&mut run, &hash);

    assert_eq!(status, "Accepted");
}
