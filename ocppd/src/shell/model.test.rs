use super::*;

#[test]
fn given_no_model_fields_when_station_built_then_a_3_phase_32_a_sim_with_one_connector() {
    let model = Model::default();

    let station = model.station("CP1", true).unwrap();

    assert_eq!(
        (station.vendor.as_str(), station.model.as_str()),
        ("ocppd", "sim")
    );
    assert_eq!(
        (station.voltage_v, station.max_current_a, station.phases),
        (230.0, 32.0, 3)
    );
    assert_eq!(station.connectors.len(), 1);
    assert_eq!(station.serial_number, "CP1");
}

#[test]
fn given_a_spawn_line_with_model_fields_when_station_built_then_it_carries_them() {
    let line = r#"{"command":"spawn","prefix":"CP","count":1,"vendor":"ACME","model":"WB22",
        "firmware_version":"1.4.2","voltage_v":120,"max_current_a":16,"phases":1,"connectors":2}"#;
    let model: Model = serde_json::from_str(line).unwrap();

    let station = model.station("CP1", false).unwrap();

    assert_eq!(
        (station.vendor.as_str(), station.model.as_str()),
        ("ACME", "WB22")
    );
    assert_eq!(station.firmware_version, "1.4.2");
    assert_eq!(
        (station.voltage_v, station.max_current_a, station.phases),
        (120.0, 16.0, 1)
    );
    assert_eq!(station.connectors.len(), 2);
}

#[test]
fn given_4_phases_when_station_built_then_refused() {
    let model = Model {
        phases: 4,
        ..Model::default()
    };

    let station = model.station("CP1", true);

    assert_eq!(station.err(), Some("phases must be 1 to 3"));
}

#[test]
fn given_no_connector_when_station_built_then_refused() {
    let model = Model {
        connectors: 0,
        ..Model::default()
    };

    let station = model.station("CP1", true);

    assert_eq!(station.err(), Some("connectors must be at least 1"));
}

#[test]
fn given_zero_current_when_station_built_then_refused() {
    let model = Model {
        max_current_a: 0.0,
        ..Model::default()
    };

    let station = model.station("CP1", true);

    assert_eq!(
        station.err(),
        Some("voltage_v and max_current_a must be positive")
    );
}

#[test]
fn given_dc_with_one_phase_when_station_built_then_refused() {
    let model = Model {
        current: Current::Dc,
        phases: 1,
        ..Model::default()
    };

    let station = model.station("CP1", true);

    assert_eq!(station.err(), Some("dc needs phases 3"));
}

#[test]
fn given_dc_spawn_line_when_station_built_then_every_connector_is_dc() {
    let line = r#"{"command":"spawn","prefix":"CP","count":1,"current":"dc","connectors":2}"#;
    let model: Model = serde_json::from_str(line).unwrap();

    let station = model.station("CP1", true).unwrap();

    assert!(
        station
            .connectors
            .iter()
            .all(|connector| connector.current == Current::Dc)
    );
}
