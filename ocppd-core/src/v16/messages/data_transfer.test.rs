use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

#[test]
fn given_data_transfer_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({"vendorId": "com.example", "messageId": "ping", "data": "{\"a\":1}"}),
        json!({"status": "UnknownVendorId"}),
    ];

    let encoded = [
        round_trip::<DataTransferRequest>(&payloads[0]),
        round_trip::<DataTransferResponse>(&payloads[1]),
    ];

    assert_eq!(encoded, payloads);
}
