use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

#[test]
fn given_local_list_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({
            "listVersion": 3, "updateType": "Differential",
            "localAuthorizationList": [
                {"idTag": "TAG1", "idTagInfo": {"status": "Blocked"}},
                {"idTag": "TAG2"}
            ]
        }),
        json!({"status": "VersionMismatch"}),
        json!({}),
        json!({"listVersion": -1}),
    ];

    let encoded = [
        round_trip::<SendLocalListRequest>(&payloads[0]),
        round_trip::<SendLocalListResponse>(&payloads[1]),
        round_trip::<GetLocalListVersionRequest>(&payloads[2]),
        round_trip::<GetLocalListVersionResponse>(&payloads[3]),
    ];

    assert_eq!(encoded, payloads);
}
