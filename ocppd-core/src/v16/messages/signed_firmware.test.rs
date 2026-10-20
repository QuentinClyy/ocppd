use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37Z";

#[test]
fn given_signed_firmware_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "retries": 3, "retryInterval": 60, "requestId": 4,
            "firmware": {
                "location": "https://example.com/fw.bin", "retrieveDateTime": TIME,
                "installDateTime": TIME, "signingCertificate": "-----BEGIN CERTIFICATE-----",
                "signature": "c2lnbmF0dXJl"
            }
        }),
        json!({"status": "RevokedCertificate"}),
        json!({"status": "InstallVerificationFailed", "requestId": 4}),
        json!({}),
    ];

    let encoded = [
        round_trip::<SignedUpdateFirmwareRequest>(&payloads[0]),
        round_trip::<SignedUpdateFirmwareResponse>(&payloads[1]),
        round_trip::<SignedFirmwareStatusNotificationRequest>(&payloads[2]),
        round_trip::<SignedFirmwareStatusNotificationResponse>(&payloads[3]),
    ];

    assert_eq!(encoded, payloads);
}
