use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37Z";

#[test]
fn given_firmware_and_diagnostics_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "location": "https://example.com/fw.bin", "retries": 3,
            "retrieveDate": TIME, "retryInterval": 60
        }),
        json!({}),
        json!({"status": "InstallationFailed"}),
        json!({}),
        json!({
            "location": "ftp://example.com/upload", "retries": 1, "retryInterval": 30,
            "startTime": TIME, "stopTime": TIME
        }),
        json!({"fileName": "diag.zip"}),
        json!({"status": "Uploading"}),
        json!({}),
    ];

    let encoded = [
        round_trip::<UpdateFirmwareRequest>(&payloads[0]),
        round_trip::<UpdateFirmwareResponse>(&payloads[1]),
        round_trip::<FirmwareStatusNotificationRequest>(&payloads[2]),
        round_trip::<FirmwareStatusNotificationResponse>(&payloads[3]),
        round_trip::<GetDiagnosticsRequest>(&payloads[4]),
        round_trip::<GetDiagnosticsResponse>(&payloads[5]),
        round_trip::<DiagnosticsStatusNotificationRequest>(&payloads[6]),
        round_trip::<DiagnosticsStatusNotificationResponse>(&payloads[7]),
    ];

    assert_eq!(encoded, payloads);
}
