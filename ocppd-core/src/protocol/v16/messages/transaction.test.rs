use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37.328Z";

#[test]
fn given_authorize_and_transaction_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({"idTag": "TAG1"}),
        json!({"idTagInfo": {
            "expiryDate": TIME, "parentIdTag": "PARENT", "status": "ConcurrentTx"
        }}),
        json!({
            "connectorId": 1, "idTag": "TAG1", "meterStart": 1200, "reservationId": 7, "timestamp": TIME
        }),
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 42}),
        json!({
            "idTag": "TAG1", "meterStop": 13200, "timestamp": TIME, "transactionId": 42,
            "reason": "EVDisconnected",
            "transactionData": [{"timestamp": TIME, "sampledValue": [{"value": "13200"}]}]
        }),
        json!({}),
    ];

    let encoded = [
        round_trip::<AuthorizeRequest>(&payloads[0]),
        round_trip::<AuthorizeResponse>(&payloads[1]),
        round_trip::<StartTransactionRequest>(&payloads[2]),
        round_trip::<StartTransactionResponse>(&payloads[3]),
        round_trip::<StopTransactionRequest>(&payloads[4]),
        round_trip::<StopTransactionResponse>(&payloads[5]),
    ];

    assert_eq!(encoded, payloads);
}
