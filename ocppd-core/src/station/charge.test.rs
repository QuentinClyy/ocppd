use super::*;

const HOUR: u64 = 3_600_000;
const STARTED: u64 = 1_000;

fn ev() -> Ev {
    Ev {
        capacity_wh: 60_000.0,
        pack_v: 400.0,
        ac_w: 11_000.0,
        ac_phases: 3,
        dc_w: 150_000.0,
        target_soc: 1.0,
    }
}

fn single_phase_ev() -> Ev {
    Ev {
        ac_phases: 1,
        ..ev()
    }
}

fn high_voltage_ev() -> Ev {
    Ev {
        pack_v: 800.0,
        dc_w: 300_000.0,
        ..ev()
    }
}

fn half_target() -> Ev {
    Ev {
        target_soc: 0.5,
        ..ev()
    }
}

fn ac(limit_w: f64) -> Charging {
    Charging {
        supply: Supply {
            current: Current::Ac,
            phases: 3,
            max_output_a: 500.0,
        },
        start: STARTED,
        start_soc: 0.2,
        start_register_wh: 5_000.0,
        limit_w,
        ramp_from: Some(STARTED),
    }
}

fn dc(limit_w: f64, max_output_a: f64) -> Charging {
    Charging {
        supply: Supply {
            current: Current::Dc,
            phases: 3,
            max_output_a,
        },
        ..ac(limit_w)
    }
}

fn running(charging: Charging) -> Charging {
    Charging {
        ramp_from: None,
        ..charging
    }
}

fn at_soc(charging: Charging, soc: f64) -> Charging {
    Charging {
        start_soc: soc,
        ..charging
    }
}

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

#[test]
fn given_ac_ev_mid_charge_when_an_hour_passes_then_its_onboard_power_is_added() {
    let session = ac(22_000.0);

    let at_hour = sample(&session, &ev(), STARTED + HOUR);

    assert!(close(at_hour.power_w, 11_000.0, 1e-6), "{at_hour:?}");
    assert!(close(at_hour.register_wh, 16_000.0, 0.01), "{at_hour:?}");
}

#[test]
fn given_station_limit_below_the_ev_when_charging_then_the_limit_caps_the_power() {
    let session = ac(7_400.0);

    let at_hour = sample(&session, &ev(), STARTED + HOUR);

    assert!(close(at_hour.power_w, 7_400.0, 1e-6));
}

#[test]
fn given_single_phase_ev_on_three_phase_station_when_charging_then_one_phase_of_power() {
    let session = ac(22_080.0);

    let at_start = sample(&session, &single_phase_ev(), STARTED);

    assert!(close(at_start.power_w, 7_360.0, 1e-6), "{at_start:?}");
}

#[test]
fn given_ac_ev_at_95_percent_when_sampled_then_still_full_onboard_power() {
    let session = at_soc(ac(22_000.0), 0.95);

    let at_start = sample(&session, &ev(), STARTED);

    assert!(close(at_start.power_w, 11_000.0, 1e-6), "{at_start:?}");
}

#[test]
fn given_ac_ev_past_99_percent_when_sampled_then_power_tapers() {
    let session = at_soc(ac(22_000.0), 0.995);

    let at_start = sample(&session, &ev(), STARTED);

    assert!(
        at_start.power_w < 11_000.0 && at_start.power_w > 0.0,
        "{at_start:?}"
    );
}

#[test]
fn given_ac_session_when_started_then_no_startup_ramp() {
    let session = ac(22_000.0);

    let at_start = sample(&session, &ev(), STARTED);

    assert!(close(at_start.power_w, 11_000.0, 1e-6));
}

#[test]
fn given_dc_ev_on_unlimited_charger_when_at_20_percent_then_its_peak_power() {
    let session = at_soc(running(dc(1e9, 1e9)), 0.2);

    let at_peak = sample(&session, &ev(), STARTED);

    assert!(close(at_peak.power_w, 150_000.0, 1e-6), "{at_peak:?}");
}

#[test]
fn given_dc_ev_on_unlimited_charger_when_soc_climbs_then_plateau_then_taper_then_drop() {
    let power_at = |soc| sample(&at_soc(running(dc(1e9, 1e9)), soc), &ev(), STARTED).power_w;

    assert!(power_at(0.5) > 0.97 * power_at(0.2));
    assert!(power_at(0.8) < 0.7 * power_at(0.2));
    assert!(power_at(0.9) < 0.6 * power_at(0.8));
    assert!(power_at(0.999) < 0.05 * power_at(0.2));
}

#[test]
fn given_dc_ev_on_current_limited_charger_when_soc_climbs_then_current_flat_and_power_rises() {
    let at = |soc| sample(&at_soc(running(dc(52_600.0, 125.0)), soc), &ev(), STARTED);

    let (low, high) = (at(0.1), at(0.3));

    assert!(close(low.current_a, 125.0, 1e-9) && close(high.current_a, 125.0, 1e-9));
    assert!(high.power_w > low.power_w);
    assert!(high.voltage_v > low.voltage_v);
}

#[test]
fn given_dc_ev_on_power_limited_charger_when_charging_then_output_is_input_after_losses() {
    let session = at_soc(running(dc(50_000.0, 500.0)), 0.3);

    let at_start = sample(&session, &ev(), STARTED);

    assert!(
        close(at_start.power_w, 50_000.0 * DC_EFFICIENCY, 1e-6),
        "{at_start:?}"
    );
}

#[test]
fn given_400_and_800_volt_evs_on_500_amp_charger_when_charging_then_800_volt_gets_twice() {
    let session = at_soc(running(dc(1e9, 500.0)), 0.3);

    let low = sample(&session, &ev(), STARTED).power_w;
    let high = sample(&session, &high_voltage_ev(), STARTED).power_w;

    assert!(close(high, 2.0 * low, 1e-6), "{low} {high}");
}

#[test]
fn given_dc_session_when_starting_then_no_current_during_cable_check_and_precharge() {
    let session = dc(150_000.0, 500.0);

    let precharge = sample(&session, &ev(), STARTED + 4_000);

    assert_eq!(precharge.current_a, 0.0);
    assert_eq!(precharge.register_wh, 5_000.0);
    assert!(precharge.voltage_v > 0.0);
}

#[test]
fn given_dc_session_when_two_seconds_into_the_ramp_then_current_is_the_ramp_rate_times_two() {
    let session = dc(150_000.0, 500.0);

    let ramping = sample(&session, &ev(), STARTED + 7_000);

    assert!(close(ramping.current_a, 100.0, 1e-9), "{ramping:?}");
}

#[test]
fn given_dc_session_when_its_limit_changes_during_the_ramp_then_the_ramp_goes_on() {
    let session = reanchor(&dc(150_000.0, 500.0), &ev(), STARTED + 6_000, 100_000.0);

    let ramping = sample(&session, &ev(), STARTED + 7_000);

    assert!(close(ramping.current_a, 100.0, 1e-9), "{ramping:?}");
}

#[test]
fn given_dc_session_two_seconds_into_the_ramp_when_sampled_then_the_register_holds_the_ramped_energy()
 {
    let session = dc(150_000.0, 500.0);

    let ramping = sample(&session, &ev(), STARTED + 7_000);

    let ramped_wh = ramping.voltage_v * RAMP_A_PER_S * 2.0 * 2.0 / 2.0 / 3_600.0;
    let charged_wh = ramping.register_wh - 5_000.0;
    assert!(
        close(charged_wh, ramped_wh, ramped_wh * 0.01),
        "{charged_wh} {ramped_wh}"
    );
}

#[test]
fn given_dc_session_when_sampled_every_10_seconds_then_register_matches_integrated_power() {
    let session = dc(150_000.0, 500.0);
    let samples: Vec<Sample> = (0..=360)
        .map(|step| sample(&session, &ev(), STARTED + step * 10_000))
        .collect();

    let integrated: f64 = samples
        .windows(2)
        .map(|pair| (pair[0].power_w + pair[1].power_w) / 2.0 * 10.0 / 3_600.0)
        .sum();

    let registered = samples[360].register_wh - samples[0].register_wh;
    assert!(
        close(integrated, registered, registered * 0.01),
        "{integrated} {registered}"
    );
}

#[test]
fn given_day_of_charging_when_sampled_every_minute_then_register_never_decreases_and_soc_stays_in_range()
 {
    let session = ac(22_000.0);

    let samples: Vec<Sample> = (0..=24 * 60)
        .map(|minute| sample(&session, &ev(), STARTED + minute * 60_000))
        .collect();

    for (minute, pair) in samples.windows(2).enumerate() {
        assert!(
            pair[1].register_wh >= pair[0].register_wh,
            "minute {minute}"
        );
        assert!(pair[1].soc <= 1.0 && pair[1].soc >= 0.2);
    }
}

#[test]
fn given_full_target_when_finish_time_computed_then_power_until_then_and_zero_at_it_and_after() {
    let session = ac(22_000.0);

    let finish = finish_time(&session, &ev()).unwrap();

    assert!(sample(&session, &ev(), finish - 1_000).power_w > 0.0);
    let done = sample(&session, &ev(), finish);
    assert_eq!(done.power_w, 0.0);
    assert!(close(done.soc, 1.0, 1e-9));
    assert_eq!(sample(&session, &ev(), finish + HOUR), done);
}

#[test]
fn given_target_on_the_flat_part_when_finish_time_computed_then_energy_over_power() {
    let session = ac(22_000.0);

    let finish = finish_time(&session, &half_target()).unwrap();

    let energy_wh = sample(&session, &half_target(), finish).register_wh - 5_000.0;
    let expected = STARTED + (energy_wh / 11_000.0 * HOUR as f64) as u64;
    assert!(finish.abs_diff(expected) <= 1, "{finish} {expected}");
    assert!(close(
        sample(&session, &half_target(), finish).soc,
        0.5,
        1e-9
    ));
}

#[test]
fn given_dc_session_when_finish_time_computed_then_sample_reaches_target_exactly_there() {
    let session = dc(150_000.0, 500.0);

    let finish = finish_time(&session, &ev()).unwrap();

    let before = sample(&session, &ev(), finish - 1_000);
    let done = sample(&session, &ev(), finish);
    assert!(before.power_w > 0.0 && before.soc < 1.0, "{before:?}");
    assert!(close(done.soc, 1.0, 1e-9));
    assert_eq!(done.power_w, 0.0);
}

#[test]
fn given_full_battery_when_finish_time_computed_then_its_start_and_no_power_after() {
    let session = at_soc(ac(22_000.0), 1.0);

    let finish = finish_time(&session, &ev());

    assert_eq!(finish, Some(STARTED));
    assert_eq!(sample(&session, &ev(), STARTED + HOUR).power_w, 0.0);
}

#[test]
fn given_zero_limit_when_finish_time_computed_then_none_and_the_register_stays() {
    let session = ac(0.0);

    let finish = finish_time(&session, &ev());

    assert_eq!(finish, None);
    assert_eq!(sample(&session, &ev(), STARTED + HOUR).register_wh, 5_000.0);
}

#[test]
fn given_dc_session_when_reanchored_midway_then_register_and_soc_are_continuous() {
    let original = dc(150_000.0, 500.0);
    let now = STARTED + HOUR / 3;

    let lowered = reanchor(&original, &ev(), now, 50_000.0);

    let before = sample(&original, &ev(), now);
    let after = sample(&lowered, &ev(), now);
    assert!(close(before.register_wh, after.register_wh, 1e-6));
    assert!(close(before.soc, after.soc, 1e-12));
    assert!(after.power_w <= 50_000.0);
}

#[test]
fn given_dc_session_when_sampled_across_the_curve_then_soc_never_decreases() {
    let session = dc(150_000.0, 500.0);

    let socs: Vec<f64> = (0..=120)
        .map(|minute| sample(&session, &ev(), STARTED + minute * 60_000).soc)
        .collect();

    assert!(socs.windows(2).all(|pair| pair[1] >= pair[0]));
}
