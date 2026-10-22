use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

#[test]
fn given_certificate_payloads_when_round_tripped_then_unchanged() {
    let hash = json!({
        "hashAlgorithm": "SHA256", "issuerNameHash": "aa11", "issuerKeyHash": "bb22",
        "serialNumber": "01"
    });
    let payloads = [
        json!({"csr": "-----BEGIN CERTIFICATE REQUEST-----"}),
        json!({"status": "Accepted"}),
        json!({"certificateChain": "-----BEGIN CERTIFICATE-----"}),
        json!({"status": "Rejected"}),
        json!({
            "certificateType": "ManufacturerRootCertificate", "certificate": "-----BEGIN CERTIFICATE-----"
        }),
        json!({"status": "Failed"}),
        json!({"certificateType": "CentralSystemRootCertificate"}),
        json!({"certificateHashData": [hash], "status": "Accepted"}),
        json!({"certificateHashData": hash}),
        json!({"status": "NotFound"}),
    ];

    let encoded = [
        round_trip::<SignCertificateRequest>(&payloads[0]),
        round_trip::<SignCertificateResponse>(&payloads[1]),
        round_trip::<CertificateSignedRequest>(&payloads[2]),
        round_trip::<CertificateSignedResponse>(&payloads[3]),
        round_trip::<InstallCertificateRequest>(&payloads[4]),
        round_trip::<InstallCertificateResponse>(&payloads[5]),
        round_trip::<GetInstalledCertificateIdsRequest>(&payloads[6]),
        round_trip::<GetInstalledCertificateIdsResponse>(&payloads[7]),
        round_trip::<DeleteCertificateRequest>(&payloads[8]),
        round_trip::<DeleteCertificateResponse>(&payloads[9]),
    ];

    assert_eq!(encoded, payloads);
}
