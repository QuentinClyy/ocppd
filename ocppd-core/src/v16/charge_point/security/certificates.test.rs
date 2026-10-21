use crate::api::event::EventKind;
use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::result;
use crate::fixtures::x509::authority::Authority;
use crate::v16::messages::security::certificates::CertificateUse;
use serde_json::{Value, json};

const ROOT: &str = crate::fixtures::x509::ROOT;
const OTHER_ROOT: &str = crate::fixtures::x509::OTHER_ROOT;
const LEAF: &str = crate::fixtures::x509::LEAF;
const NOW: u64 = 1_798_761_600_000;
const EXPIRED: u64 = 4_954_435_200_000;
const CENTRAL: &str = "CentralSystemRootCertificate";

fn openssl_ocsp_sha256_of_root() -> Value {
    json!({
        "hashAlgorithm": "SHA256",
        "issuerNameHash": "d5c9bab6d9aeffc33047fd5069023ec4f45341a0152f8cc9fa44699ad4305f40",
        "issuerKeyHash": "e7c27d1606a8058e25e60b93a03b33cf2e58d92b119683d07a95ba55f846f409",
        "serialNumber": "143b93eb80ee17dc18bb74cc9a20cbb6cc984a9f"
    })
}

fn install(csms: &mut Csms, now: u64, certificate_type: &str, pem: &str) -> Value {
    let request = json!({"certificateType": certificate_type, "certificate": pem});
    result(csms.call(now, "InstallCertificate", request))["status"].clone()
}

fn installed(csms: &mut Csms, certificate_type: &str) -> Value {
    let request = json!({"certificateType": certificate_type});
    result(csms.call(NOW, "GetInstalledCertificateIds", request))
}

fn upper(hash_data: Value) -> Value {
    let mut upper = hash_data;
    for field in ["issuerNameHash", "issuerKeyHash", "serialNumber"] {
        upper[field] = json!(upper[field].as_str().unwrap().to_uppercase());
    }
    upper
}

fn with_root() -> Csms {
    let mut csms = Csms::booted();
    install(&mut csms, NOW, CENTRAL, ROOT);
    csms.events.clear();
    csms
}

fn full_store() -> Csms {
    let mut csms = Csms::booted();
    for serial in 1..=10 {
        let root = Authority::new(&format!("root {serial}")).pem();
        install(&mut csms, NOW, CENTRAL, &root);
    }
    csms
}

fn delete(csms: &mut Csms, hash_data: Value) -> Value {
    let request = json!({"certificateHashData": hash_data});
    result(csms.call(NOW, "DeleteCertificate", request))["status"].clone()
}

#[test]
fn given_booted_charge_point_when_root_installed_then_accepted_and_reported() {
    let mut csms = Csms::booted();

    let status = install(&mut csms, NOW, CENTRAL, ROOT);

    assert_eq!(status, "Accepted");
    let certificate_type = CertificateUse::CentralSystemRootCertificate;
    assert_eq!(
        csms.events,
        vec![EventKind::CertificateInstalled { certificate_type }]
    );
}

#[test]
fn given_installed_root_when_ids_requested_then_listed_by_its_ocsp_hash_data() {
    let mut csms = with_root();

    let listed = installed(&mut csms, CENTRAL);

    assert_eq!(listed["status"], "Accepted");
    assert_eq!(
        listed["certificateHashData"],
        json!([openssl_ocsp_sha256_of_root()])
    );
}

#[test]
fn given_root_with_a_short_serial_when_ids_requested_then_serial_without_leading_zeroes() {
    let mut csms = Csms::booted();
    install(&mut csms, NOW, CENTRAL, OTHER_ROOT);

    let listed = installed(&mut csms, CENTRAL);

    let serial = &listed["certificateHashData"][0]["serialNumber"];
    assert_eq!(serial, "e63d4eedeab0120181efcf62ec281305f0787e9");
}

#[test]
fn given_no_roots_when_ids_requested_then_not_found() {
    let mut csms = Csms::booted();

    let listed = installed(&mut csms, CENTRAL);

    assert_eq!(listed["status"], "NotFound");
}

#[test]
fn given_manufacturer_root_when_ids_requested_by_type_then_listed_under_its_type_only() {
    let mut csms = Csms::booted();
    install(&mut csms, NOW, "ManufacturerRootCertificate", ROOT);

    let listed = [
        installed(&mut csms, CENTRAL),
        installed(&mut csms, "ManufacturerRootCertificate"),
    ];

    assert_eq!(listed[0], json!({"status": "NotFound"}));
    assert_eq!(listed[1]["status"], "Accepted");
}

#[test]
fn given_booted_charge_point_when_invalid_certificates_installed_then_rejected_and_nothing_stored()
{
    let mut csms = Csms::booted();

    let statuses = [
        install(&mut csms, NOW, CENTRAL, "garbage"),
        install(&mut csms, NOW, CENTRAL, LEAF),
        install(&mut csms, 0, CENTRAL, ROOT),
        install(&mut csms, EXPIRED, CENTRAL, ROOT),
    ];

    assert_eq!(statuses, ["Rejected"; 4], "garbage, leaf, not yet, expired");
    assert_eq!(installed(&mut csms, CENTRAL)["status"], "NotFound");
    assert!(csms.events.is_empty());
}

#[test]
fn given_installed_root_when_installed_again_then_accepted_and_one_copy_kept() {
    let mut csms = with_root();

    let status = install(&mut csms, NOW, CENTRAL, ROOT);

    assert_eq!(status, "Accepted");
    let listed = installed(&mut csms, CENTRAL);
    assert_eq!(listed["certificateHashData"].as_array().unwrap().len(), 1);
}

#[test]
fn given_booted_charge_point_when_ten_roots_installed_then_all_accepted() {
    let mut csms = Csms::booted();

    let statuses: Vec<Value> = (1..=10)
        .map(|serial| {
            let root = Authority::new(&format!("root {serial}")).pem();
            install(&mut csms, NOW, CENTRAL, &root)
        })
        .collect();

    assert_eq!(statuses, vec![json!("Accepted"); 10]);
}

#[test]
fn given_certificate_store_max_length_roots_when_another_installed_then_rejected() {
    let mut csms = full_store();

    let status = install(&mut csms, NOW, CENTRAL, &Authority::new("root 11").pem());

    assert_eq!(status, "Rejected");
}

#[test]
fn given_installed_root_when_deleted_by_uppercase_hash_data_then_accepted_removed_and_reported() {
    let mut csms = with_root();

    let status = delete(&mut csms, upper(openssl_ocsp_sha256_of_root()));

    assert_eq!(status, "Accepted");
    assert_eq!(installed(&mut csms, CENTRAL)["status"], "NotFound");
    let certificate_type = CertificateUse::CentralSystemRootCertificate;
    assert_eq!(
        csms.events,
        vec![EventKind::CertificateDeleted { certificate_type }]
    );
}

#[test]
fn given_deleted_root_when_deleted_again_then_not_found() {
    let mut csms = with_root();
    delete(&mut csms, openssl_ocsp_sha256_of_root());

    let status = delete(&mut csms, openssl_ocsp_sha256_of_root());

    assert_eq!(status, "NotFound");
}

#[test]
fn given_installed_root_when_deleted_by_sha256_values_labeled_sha512_then_not_found() {
    let mut csms = with_root();
    let mut mislabeled = openssl_ocsp_sha256_of_root();
    mislabeled["hashAlgorithm"] = json!("SHA512");

    let status = delete(&mut csms, mislabeled);

    assert_eq!(status, "NotFound", "SHA-256 values");
}

#[test]
fn given_installed_root_when_deleted_by_its_sha512_hash_data_then_accepted() {
    let mut csms = with_root();
    let openssl_ocsp_sha512_of_root = json!({
        "hashAlgorithm": "SHA512",
        "issuerNameHash": "f962580a3612077af01e4ee033a4224869cd4e0dac2da10ce9d032df47564c82374d25b859e19c33c4f52a9de7290353f94be2c2a9706e44f7b0b2ab8565eee1",
        "issuerKeyHash": "f11011456da0ca0081b12ff4d5705413d7bba85c39289fc910f3a926013c44aa384a66460401db210ae9013996c8de699b650f7376008341bd3abbaacd380d12",
        "serialNumber": "143b93eb80ee17dc18bb74cc9a20cbb6cc984a9f"
    });

    let status = delete(&mut csms, openssl_ocsp_sha512_of_root);

    assert_eq!(status, "Accepted");
}
