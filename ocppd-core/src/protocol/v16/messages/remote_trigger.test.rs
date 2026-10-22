use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

#[test]
fn given_trigger_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({"requestedMessage": "DiagnosticsStatusNotification", "connectorId": 1}),
        json!({"status": "NotImplemented"}),
        json!({"requestedMessage": "SignChargePointCertificate"}),
        json!({"status": "Accepted"}),
    ];

    let encoded = [
        round_trip::<TriggerMessageRequest>(&payloads[0]),
        round_trip::<TriggerMessageResponse>(&payloads[1]),
        round_trip::<ExtendedTriggerMessageRequest>(&payloads[2]),
        round_trip::<ExtendedTriggerMessageResponse>(&payloads[3]),
    ];

    assert_eq!(encoded, payloads);
}
