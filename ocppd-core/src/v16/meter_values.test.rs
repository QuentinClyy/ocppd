use crate::event::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, register};
use crate::intent::EvSpec;
use crate::v16::messages::status::ChargePointStatus;
use serde_json::{Value, json};

fn three_minutes() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    for minute in 1..=3 {
        csms.input(minute * MINUTE, Input::Tick);
    }
    csms
}

fn first_sample() -> Value {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.input(MINUTE, Input::Tick);
    csms.sent("MeterValues")[0]["meterValue"][0]["sampledValue"].clone()
}

fn small_battery() -> Csms {
    let small = EvSpec {
        capacity_kwh: 0.5,
        soc: 0.99,
        ..EvSpec::default()
    };
    Csms::charging(0, small)
}

#[test]
fn given_charging_ev_when_meter_values_sent_then_due_every_minute() {
    let mut csms = Csms::charging(0, EvSpec::default());

    let deadlines: Vec<Option<u64>> = (1..=3)
        .map(|minute| {
            let deadline = csms.charge_point.next_deadline();
            csms.input(minute * MINUTE, Input::Tick);
            deadline
        })
        .collect();

    assert_eq!(
        deadlines,
        vec![Some(MINUTE), Some(2 * MINUTE), Some(3 * MINUTE)]
    );
}

#[test]
fn given_three_minutes_of_charging_when_meter_values_read_then_the_register_grows() {
    let csms = three_minutes();

    let registers: Vec<f64> = csms.sent("MeterValues").iter().map(register).collect();

    assert_eq!(registers, vec![183.0, 367.0, 550.0]);
}

#[test]
fn given_charging_ev_when_first_meter_values_sent_then_power_sampled_periodically_at_the_outlet() {
    let sampled = first_sample();

    let power = &sampled[1];

    assert_eq!(
        power,
        &json!({"value": "11000", "context": "Sample.Periodic",
        "measurand": "Power.Active.Import", "location": "Outlet", "unit": "W"})
    );
}

#[test]
fn given_nearly_full_battery_when_charging_then_due_to_finish_before_the_first_sample() {
    let csms = small_battery();

    let finish = csms.charge_point.next_deadline().unwrap();

    assert!(
        finish < MINUTE,
        "finish {finish} comes before the first sample"
    );
}

#[test]
fn given_nearly_full_battery_when_the_finish_time_comes_then_suspended_ev_and_next_sample_due() {
    let mut csms = small_battery();
    let finish = csms.charge_point.next_deadline().unwrap();

    csms.input(finish, Input::Tick);

    assert_eq!(
        csms.statuses().last(),
        Some(&ChargePointStatus::SuspendedEv)
    );
    assert_eq!(csms.charge_point.next_deadline(), Some(MINUTE));
}
