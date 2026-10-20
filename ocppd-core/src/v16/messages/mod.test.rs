use super::log::*;
use super::security_event::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37Z";

#[test]
fn given_security_event_and_log_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "type": "FirmwareUpdated", "timestamp": TIME, "techInfo": "0.1.0 -> 0.2.0"
        }),
        json!({}),
        json!({
            "log": {"remoteLocation": "https://example.com/logs", "oldestTimestamp": TIME,
                    "latestTimestamp": TIME},
            "logType": "SecurityLog", "requestId": 3, "retries": 2, "retryInterval": 10
        }),
        json!({"status": "AcceptedCanceled", "filename": "security.log"}),
        json!({"status": "UploadFailure", "requestId": 3}),
        json!({}),
    ];

    let encoded = [
        round_trip::<SecurityEventNotificationRequest>(&payloads[0]),
        round_trip::<SecurityEventNotificationResponse>(&payloads[1]),
        round_trip::<GetLogRequest>(&payloads[2]),
        round_trip::<GetLogResponse>(&payloads[3]),
        round_trip::<LogStatusNotificationRequest>(&payloads[4]),
        round_trip::<LogStatusNotificationResponse>(&payloads[5]),
    ];

    assert_eq!(encoded, payloads);
}
