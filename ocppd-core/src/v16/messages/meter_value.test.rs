use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37.328Z";

#[test]
fn given_meter_values_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "connectorId": 1, "transactionId": 42,
            "meterValue": [{"timestamp": TIME, "sampledValue": [
                {"value": "5000", "context": "Sample.Periodic", "format": "Raw",
                 "measurand": "Energy.Active.Import.Register", "location": "Outlet", "unit": "Wh"},
                {"value": "16.0", "measurand": "Current.Import", "phase": "L1-N", "unit": "A"},
                {"value": "80", "measurand": "SoC", "location": "EV", "unit": "Percent"},
                {"value": "11.0", "measurand": "Power.Active.Import", "unit": "kW"}
            ]}]
        }),
        json!({}),
    ];

    let encoded = [
        round_trip::<MeterValuesRequest>(&payloads[0]),
        round_trip::<MeterValuesResponse>(&payloads[1]),
    ];

    assert_eq!(encoded, payloads);
}
