use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

const TIME: &str = "2026-10-20T03:50:37Z";

#[test]
fn given_reservation_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "connectorId": 1, "expiryDate": TIME, "idTag": "TAG1",
            "parentIdTag": "PARENT", "reservationId": 9
        }),
        json!({"status": "Occupied"}),
        json!({"reservationId": 9}),
        json!({"status": "Rejected"}),
    ];

    let encoded = [
        round_trip::<ReserveNowRequest>(&payloads[0]),
        round_trip::<ReserveNowResponse>(&payloads[1]),
        round_trip::<CancelReservationRequest>(&payloads[2]),
        round_trip::<CancelReservationResponse>(&payloads[3]),
    ];

    assert_eq!(encoded, payloads);
}
