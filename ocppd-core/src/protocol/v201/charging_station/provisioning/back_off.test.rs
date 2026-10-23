use crate::fixtures::v201::station::Run;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::station::Station;
use crate::station::connector::Connector;

fn configure(station: &mut ChargingStation, wait: &str, repeat: &str, range: &str) {
    for (variable, value) in [
        ("RetryBackOffWaitMinimum", wait),
        ("RetryBackOffRepeatTimes", repeat),
        ("RetryBackOffRandomRange", range),
    ] {
        assert!(station.configure(&format!("OCPPCommCtrlr.{variable}"), value));
    }
}

#[test]
fn given_back_off_of_10_s_doubled_twice_when_connections_fail_then_waits_double_up_to_the_cap_per_part_4_5_4_and_tc_b_57()
 {
    let mut run = Run::new(1);
    configure(&mut run.station, "10", "2", "0");

    let delays: Vec<_> = (1..=5)
        .map(|failures| run.station.reconnect_delay_ms(failures))
        .collect();

    assert_eq!(delays, vec![10_000, 20_000, 40_000, 40_000, 40_000]);
}

#[test]
fn given_random_range_of_5_s_when_connections_fail_then_each_wait_adds_a_random_part_within_it_per_part_4_5_4()
 {
    let station = Station {
        id: "CS1".into(),
        vendor: "ocppd".into(),
        model: "sim".into(),
        serial_number: "SN-1".into(),
        firmware_version: "0.1.0".into(),
        voltage_v: 230.0,
        max_current_a: 32.0,
        phases: 3,
        jitter: true,
        connectors: vec![Connector::default()],
    };
    let mut station = ChargingStation::new(station).unwrap();
    configure(&mut station, "10", "0", "5");

    let delays: Vec<_> = (1..=20)
        .map(|failures| station.reconnect_delay_ms(failures))
        .collect();

    assert!(
        delays.iter().all(|delay| (10_000..15_000).contains(delay)),
        "{delays:?}"
    );
    assert!(
        delays.windows(2).any(|pair| pair[0] != pair[1]),
        "{delays:?}"
    );
}
