use super::boot::*;
use super::meter_values::*;
use super::status::*;
use super::transaction::*;
use crate::fixtures::round_trip::decodes;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37.328Z";

#[test]
fn given_out_of_spec_payloads_when_decoded_then_refused() {
    let payloads = [
        json!({
            "connectorId": 1,
            "meterValue": [{"timestamp": TIME, "sampledValue": [{"value": "1", "unit": "Hertz"}]}]
        }),
        json!({"idTag": "an id tag over 20 chars"}),
        json!({"status": "Accepted", "currentTime": "yesterday", "interval": 60}),
        json!({"connectorId": 1, "errorCode": "NoError", "status": "Sleeping"}),
    ];

    let decoded = [
        decodes::<MeterValuesRequest>(&payloads[0]),
        decodes::<AuthorizeRequest>(&payloads[1]),
        decodes::<BootNotificationResponse>(&payloads[2]),
        decodes::<StatusNotificationRequest>(&payloads[3]),
    ];

    assert_eq!(decoded, [false; 4]);
}
