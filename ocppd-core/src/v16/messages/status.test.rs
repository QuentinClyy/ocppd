use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37.328Z";

#[test]
fn given_status_notification_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "connectorId": 1, "errorCode": "EVCommunicationError", "info": "pilot lost",
            "status": "SuspendedEVSE", "timestamp": TIME, "vendorId": "ocppd", "vendorErrorCode": "E42"
        }),
        json!({"connectorId": 0, "errorCode": "NoError", "status": "Available"}),
        json!({}),
    ];

    let encoded = [
        round_trip::<StatusNotificationRequest>(&payloads[0]),
        round_trip::<StatusNotificationRequest>(&payloads[1]),
        round_trip::<StatusNotificationResponse>(&payloads[2]),
    ];

    assert_eq!(encoded, payloads);
}
