use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37Z";

#[test]
fn given_smart_charging_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "connectorId": 1,
            "csChargingProfiles": {
                "chargingProfileId": 5, "transactionId": 42, "stackLevel": 1,
                "chargingProfilePurpose": "TxDefaultProfile", "chargingProfileKind": "Recurring",
                "recurrencyKind": "Daily", "validFrom": TIME, "validTo": TIME,
                "chargingSchedule": {
                    "duration": 86400, "startSchedule": TIME, "chargingRateUnit": "W",
                    "chargingSchedulePeriod": [
                        {"startPeriod": 0, "limit": 11000.0, "numberPhases": 3},
                        {"startPeriod": 3600, "limit": 7400.5}
                    ],
                    "minChargingRate": 1400.0
                }
            }
        }),
        json!({"status": "NotSupported"}),
        json!({
            "id": 5, "connectorId": 1, "chargingProfilePurpose": "ChargePointMaxProfile", "stackLevel": 0
        }),
        json!({}),
        json!({"status": "Unknown"}),
        json!({"connectorId": 1, "duration": 3600, "chargingRateUnit": "A"}),
        json!({
            "status": "Accepted", "connectorId": 1, "scheduleStart": TIME,
            "chargingSchedule": {"chargingRateUnit": "A",
                "chargingSchedulePeriod": [{"startPeriod": 0, "limit": 32.0}]}
        }),
    ];

    let encoded = [
        round_trip::<SetChargingProfileRequest>(&payloads[0]),
        round_trip::<SetChargingProfileResponse>(&payloads[1]),
        round_trip::<ClearChargingProfileRequest>(&payloads[2]),
        round_trip::<ClearChargingProfileRequest>(&payloads[3]),
        round_trip::<ClearChargingProfileResponse>(&payloads[4]),
        round_trip::<GetCompositeScheduleRequest>(&payloads[5]),
        round_trip::<GetCompositeScheduleResponse>(&payloads[6]),
    ];

    assert_eq!(encoded, payloads);
}
