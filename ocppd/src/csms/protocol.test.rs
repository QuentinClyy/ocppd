use super::*;
use crate::shell::model::Model;
use serde_json::Value;

fn described(version: Version) -> Value {
    let station = Model::default().station("CS1", false).unwrap();
    let mut protocol = Protocol::new(version, station).unwrap();

    let line = protocol.description("ws://csms/CS1", 1);

    serde_json::from_str(&line).unwrap()
}

#[test]
fn given_a_2_0_1_station_when_described_then_its_model_csms_and_device_model_variables() {
    let description = described(Version::V201);

    assert_eq!(description["kind"], "station_described");
    assert_eq!(description["station"], "CS1");
    assert_eq!(description["ocpp"], "2.0.1");
    assert_eq!(
        (&description["vendor"], &description["model"]),
        (&Value::from("ocppd"), &Value::from("sim"))
    );
    assert_eq!(description["csms"], "ws://csms/CS1");
    let keys: Vec<&str> = description["configuration"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|setting| setting["key"].as_str())
        .collect();
    assert!(keys.contains(&"OCPPCommCtrlr.HeartbeatInterval"));
}

#[test]
fn given_a_1_6_station_when_described_then_its_firmware_and_configuration_keys() {
    let description = described(Version::V16);

    assert_eq!(description["ocpp"], "1.6");
    assert_eq!(description["firmware_version"], env!("CARGO_PKG_VERSION"));
    assert!(description["configuration"].as_array().unwrap().len() > 30);
}
