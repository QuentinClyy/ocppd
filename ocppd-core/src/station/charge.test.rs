use super::*;

const HOUR: u64 = 3_600_000;

fn ev() -> Ev {
    Ev {
        capacity_wh: 60_000.0,
        max_w: 11_000.0,
        target_soc: 1.0,
    }
}

fn charging(limit_w: f64) -> Charging {
    Charging {
        start: 1_000,
        start_soc: 0.2,
        start_register_wh: 5_000.0,
        limit_w,
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn knee() -> u64 {
    1_000 + (0.6 * 60_000.0 / 11_000.0 * HOUR as f64) as u64
}

fn half_target() -> Ev {
    Ev {
        target_soc: 0.5,
        ..ev()
    }
}

fn full() -> Charging {
    Charging {
        start_soc: 1.0,
        ..charging(22_000.0)
    }
}

#[test]
fn given_ev_below_the_knee_when_an_hour_passes_then_its_full_power_added() {
    let session = charging(22_000.0);

    let at_hour = sample(&session, &ev(), 1_000 + HOUR);

    assert!(close(at_hour.register_wh, 16_000.0), "{at_hour:?}");
    assert!(close(at_hour.power_w, 11_000.0));
    assert!(close(at_hour.soc, 0.2 + 11.0 / 60.0));
}

#[test]
fn given_station_limit_below_the_ev_when_an_hour_passes_then_the_limit_caps_the_power() {
    let session = charging(7_400.0);

    let at_hour = sample(&session, &ev(), 1_000 + HOUR);

    assert!(close(at_hour.power_w, 7_400.0));
    assert!(close(at_hour.register_wh, 12_400.0));
}

#[test]
fn given_ev_charging_when_it_reaches_the_knee_then_still_full_power() {
    let session = charging(22_000.0);

    let at_knee = sample(&session, &ev(), knee());

    assert!(close(at_knee.power_w, 11_000.0));
}

#[test]
fn given_ev_past_the_knee_when_an_hour_passes_then_power_tapers() {
    let session = charging(22_000.0);

    let later = sample(&session, &ev(), knee() + HOUR);

    assert!(later.power_w < 11_000.0 && later.power_w > 0.0, "{later:?}");
    assert!(later.soc > 0.8 && later.soc < 1.0);
}

#[test]
fn given_day_of_charging_when_sampled_every_minute_then_register_never_decreases_and_soc_stays_in_range()
 {
    let session = charging(22_000.0);

    let samples: Vec<Sample> = (0..=24 * 60)
        .map(|minute| sample(&session, &ev(), 1_000 + minute * 60_000))
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
    let session = charging(22_000.0);

    let finish = finish_time(&session, &ev()).unwrap();

    assert!(sample(&session, &ev(), finish - 1_000).power_w > 0.0);
    let done = sample(&session, &ev(), finish);
    assert_eq!(done.power_w, 0.0);
    assert!(close(done.soc, 0.996));
    assert_eq!(sample(&session, &ev(), finish + HOUR), done);
}

#[test]
fn given_target_below_the_knee_when_finish_time_computed_then_it_stops_at_the_target() {
    let session = charging(22_000.0);

    let finish = finish_time(&session, &half_target()).unwrap();

    let expected = 1_000 + (0.3 * 60_000.0 / 11_000.0 * HOUR as f64).ceil() as u64;
    assert_eq!(finish, expected);
    let done = sample(&session, &half_target(), finish);
    assert!(close(done.soc, 0.5));
    assert_eq!(done.power_w, 0.0);
}

#[test]
fn given_full_battery_when_finish_time_computed_then_its_start_and_no_power_after() {
    let session = full();

    let finish = finish_time(&session, &ev());

    assert_eq!(finish, Some(1_000));
    assert_eq!(sample(&session, &ev(), 1_000 + HOUR).power_w, 0.0);
}

#[test]
fn given_zero_limit_when_finish_time_computed_then_none_and_the_register_stays() {
    let session = charging(0.0);

    let finish = finish_time(&session, &ev());

    assert_eq!(finish, None);
    assert!(close(
        sample(&session, &ev(), 1_000 + HOUR).register_wh,
        5_000.0
    ));
}

#[test]
fn given_five_hours_of_charging_when_reanchored_to_a_lower_limit_then_continuous_and_capped() {
    let original = charging(22_000.0);
    let now = 1_000 + 5 * HOUR;

    let lowered = reanchor(&original, &ev(), now, 3_700.0);

    let before = sample(&original, &ev(), now);
    let after = sample(&lowered, &ev(), now);
    assert!(close(before.register_wh, after.register_wh));
    assert!(close(before.soc, after.soc));
    assert!(after.power_w <= 3_700.0);
}
