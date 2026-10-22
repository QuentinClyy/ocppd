use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

#[test]
fn given_remote_control_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "connectorId": 1, "idTag": "TAG1",
            "chargingProfile": {
                "chargingProfileId": 1, "stackLevel": 0, "chargingProfilePurpose": "TxProfile",
                "chargingProfileKind": "Relative",
                "chargingSchedule": {"chargingRateUnit": "A",
                    "chargingSchedulePeriod": [{"startPeriod": 0, "limit": 16.0}]}
            }
        }),
        json!({"status": "Accepted"}),
        json!({"transactionId": 42}),
        json!({"status": "Rejected"}),
        json!({"type": "Hard"}),
        json!({"status": "Accepted"}),
        json!({"connectorId": 1}),
        json!({"status": "UnlockFailed"}),
        json!({"connectorId": 0, "type": "Inoperative"}),
        json!({"status": "Scheduled"}),
        json!({}),
        json!({"status": "Accepted"}),
    ];

    let encoded = [
        round_trip::<RemoteStartTransactionRequest>(&payloads[0]),
        round_trip::<RemoteStartTransactionResponse>(&payloads[1]),
        round_trip::<RemoteStopTransactionRequest>(&payloads[2]),
        round_trip::<RemoteStopTransactionResponse>(&payloads[3]),
        round_trip::<ResetRequest>(&payloads[4]),
        round_trip::<ResetResponse>(&payloads[5]),
        round_trip::<UnlockConnectorRequest>(&payloads[6]),
        round_trip::<UnlockConnectorResponse>(&payloads[7]),
        round_trip::<ChangeAvailabilityRequest>(&payloads[8]),
        round_trip::<ChangeAvailabilityResponse>(&payloads[9]),
        round_trip::<ClearCacheRequest>(&payloads[10]),
        round_trip::<ClearCacheResponse>(&payloads[11]),
    ];

    assert_eq!(encoded, payloads);
}
