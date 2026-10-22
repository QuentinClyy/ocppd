use super::*;
use serde_json::json;

#[test]
fn given_custom_data_at_any_depth_when_stripped_then_dropped() {
    let payload = json!({
        "customData": {"vendorId": "x"},
        "a": [{"b": 1, "customData": {"vendorId": "y", "extra": true}}]
    });

    let stripped = without_custom_data(payload);

    assert_eq!(stripped, json!({"a": [{"b": 1}]}));
}
