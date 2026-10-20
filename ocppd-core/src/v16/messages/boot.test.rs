use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37.328Z";

#[test]
fn given_boot_and_heartbeat_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "chargePointVendor": "ocppd", "chargePointModel": "sim",
            "chargePointSerialNumber": "CP1", "chargeBoxSerialNumber": "BOX1",
            "firmwareVersion": "0.1.0", "iccid": "8931440000000000000", "imsi": "310150123456789",
            "meterType": "sim-meter", "meterSerialNumber": "M1"
        }),
        json!({"chargePointVendor": "v", "chargePointModel": "m"}),
        json!({"status": "Pending", "currentTime": TIME, "interval": 14400}),
        json!({}),
        json!({"currentTime": TIME}),
    ];

    let encoded = [
        round_trip::<BootNotificationRequest>(&payloads[0]),
        round_trip::<BootNotificationRequest>(&payloads[1]),
        round_trip::<BootNotificationResponse>(&payloads[2]),
        round_trip::<HeartbeatRequest>(&payloads[3]),
        round_trip::<HeartbeatResponse>(&payloads[4]),
    ];

    assert_eq!(encoded, payloads);
}
