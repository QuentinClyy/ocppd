use super::*;
use crate::fixtures::round_trip::round_trip;
use serde_json::json;

#[test]
fn given_configuration_payloads_when_round_tripped_then_unchanged() {
    let payloads = [
        json!({"key": "HeartbeatInterval", "value": "60"}),
        json!({"status": "RebootRequired"}),
        json!({"key": ["HeartbeatInterval"]}),
        json!({}),
        json!({
            "configurationKey": [{"key": "HeartbeatInterval", "readonly": false, "value": "60"}],
            "unknownKey": ["Nope"]
        }),
    ];

    let encoded = [
        round_trip::<ChangeConfigurationRequest>(&payloads[0]),
        round_trip::<ChangeConfigurationResponse>(&payloads[1]),
        round_trip::<GetConfigurationRequest>(&payloads[2]),
        round_trip::<GetConfigurationRequest>(&payloads[3]),
        round_trip::<GetConfigurationResponse>(&payloads[4]),
    ];

    assert_eq!(encoded, payloads);
}
