use super::*;
use crate::fixtures::v16::raw::station;
use crate::station::charge::Ev;

const NOW: u64 = 60_000;

fn ev(ac_phases: u32) -> Ev {
    Ev {
        capacity_wh: 60_000.0,
        pack_v: 400.0,
        ac_w: 11_000.0,
        ac_phases,
        dc_w: 150_000.0,
        target_soc: 1.0,
    }
}

fn plugged(current: Current, ac_phases: u32, jitter: bool) -> Station {
    let mut station = Station {
        jitter,
        ..station()
    };
    station.connectors[0].current = current;
    station.connectors[0].plug_in(ev(ac_phases), 0.3);
    station
}

fn charging_sample(power_w: f64) -> Sample {
    Sample {
        register_wh: 0.0,
        power_w,
        current_a: power_w / 400.0,
        voltage_v: 400.0,
        soc: 0.3,
    }
}

#[test]
fn given_single_phase_ev_on_three_phase_station_when_charging_then_current_on_l1_only() {
    let station = plugged(Current::Ac, 1, false);

    let lines = lines(&station, 0, &charging_sample(7_360.0), NOW);

    let amps: Vec<f64> = lines.iter().map(|line| line.current_a).collect();
    assert_eq!(amps, vec![32.0, 0.0, 0.0]);
    assert!(lines.iter().all(|line| line.voltage_v == 230.0));
}

#[test]
fn given_three_phase_ev_when_charging_then_current_split_over_three_phases() {
    let station = plugged(Current::Ac, 3, false);

    let lines = lines(&station, 0, &charging_sample(11_040.0), NOW);

    assert!(
        lines
            .iter()
            .all(|line| (line.current_a - 16.0).abs() < 1e-9)
    );
    assert_eq!(power_w(&lines, Side::Outlet), Some(11_040.0));
    assert_eq!(power_w(&lines, Side::Inlet), None);
}

#[test]
fn given_dc_station_when_charging_then_dc_outlet_and_three_phase_inlet_after_losses() {
    let station = plugged(Current::Dc, 3, false);

    let lines = lines(&station, 0, &charging_sample(95_000.0), NOW);

    let outlet = lines[0];
    assert_eq!((outlet.side, outlet.phase), (Side::Outlet, None));
    assert_eq!((outlet.voltage_v, outlet.current_a), (400.0, 237.5));
    let inlet = power_w(&lines, Side::Inlet).unwrap();
    assert!((inlet - 100_000.0).abs() < 1e-6, "{inlet}");
    assert_eq!(lines.len(), 4);
}

#[test]
fn given_jitter_when_sampled_then_every_reading_within_its_noise_and_phases_differ() {
    let station = plugged(Current::Ac, 3, true);

    let lines = lines(&station, 0, &charging_sample(11_040.0), NOW);

    assert!(
        lines
            .iter()
            .all(|line| (line.voltage_v - 230.0).abs() <= 230.0 * VOLTAGE_NOISE)
    );
    assert!(
        lines
            .iter()
            .all(|line| (line.current_a - 16.0).abs() <= 16.0 * CURRENT_NOISE)
    );
    assert_ne!(lines[0].current_a, lines[1].current_a);
}

#[test]
fn given_jitter_when_sampled_twice_at_the_same_time_then_same_readings() {
    let station = plugged(Current::Dc, 3, true);

    let first = lines(&station, 0, &charging_sample(95_000.0), NOW);

    assert_eq!(lines(&station, 0, &charging_sample(95_000.0), NOW), first);
}

#[test]
fn given_dc_station_when_offering_power_then_offered_current_is_at_battery_voltage_capped() {
    let station = plugged(Current::Dc, 3, false);

    let offered = offered_a(&station, 0, &charging_sample(1.0), 400_000.0);

    assert_eq!(offered, 500.0);
}

#[test]
fn given_single_phase_ev_on_three_phase_station_when_offering_then_offered_current_is_per_station_phase()
 {
    let station = plugged(Current::Ac, 1, false);

    let offered = offered_a(&station, 0, &charging_sample(7_360.0), 22_080.0);

    assert_eq!(offered, 32.0);
}

#[test]
fn given_jittered_station_with_two_connectors_when_both_sampled_at_once_then_their_noise_differs() {
    let mut station = plugged(Current::Ac, 3, true);
    station.connectors.push(station.connectors[0].clone());

    let first = lines(&station, 0, &charging_sample(11_040.0), NOW);
    let second = lines(&station, 1, &charging_sample(11_040.0), NOW);

    assert_ne!(first, second);
}

#[test]
fn given_jitter_when_the_station_is_the_limit_then_no_side_draws_more_than_offered() {
    let mut station = plugged(Current::Dc, 3, true);
    let offered_w = 100_000.0;
    let supply = station.supply(0);
    station.connectors[0].start_charging(0, supply, offered_w);
    let sample = charging_sample(offered_w * DC_EFFICIENCY);

    for now in (0..300).map(|second| second * 1_000) {
        let lines = lines(&station, 0, &sample, now);

        for side in [Side::Outlet, Side::Inlet] {
            let watts = power_w(&lines, side).unwrap().round();
            assert!(watts <= offered_w, "{side:?} {watts} W at {now} ms");
        }
    }
}
