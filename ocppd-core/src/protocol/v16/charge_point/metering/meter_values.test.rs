use crate::api::intent::EvSpec;
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, register};
use crate::protocol::v16::messages::availability::ChargePointStatus;
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

fn per_phase(sampled: &Value, measurand: &str) -> Vec<(String, String)> {
    sampled
        .as_array()
        .unwrap()
        .iter()
        .filter(|value| value["measurand"] == measurand)
        .map(|value| {
            (
                value["phase"].as_str().unwrap().into(),
                value["value"].as_str().unwrap().into(),
            )
        })
        .collect()
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
fn given_charging_ev_when_first_meter_values_sent_then_current_and_voltage_per_phase() {
    let sampled = first_sample();

    let readings = [
        per_phase(&sampled, "Current.Import"),
        per_phase(&sampled, "Voltage"),
    ];

    let amps = |phase: &str| (phase.to_string(), "15.9".to_string());
    let volts = |phase: &str| (phase.to_string(), "230.0".to_string());
    assert_eq!(readings[0], vec![amps("L1"), amps("L2"), amps("L3")]);
    assert_eq!(
        readings[1],
        vec![volts("L1-N"), volts("L2-N"), volts("L3-N")]
    );
}

#[test]
fn given_charging_ev_when_interval_and_offered_measurands_configured_then_accepted() {
    let mut csms = Csms::charging(0, EvSpec::default());
    let measurands = "Energy.Active.Import.Interval,Power.Offered,Current.Offered";

    let status = csms.configure(0, "MeterValuesSampledData", measurands);

    assert_eq!(status, "Accepted");
}

#[test]
fn given_interval_and_offered_measurands_when_two_samples_sent_then_interval_energy_and_offered_limits()
 {
    let mut csms = Csms::charging(0, EvSpec::default());
    let measurands = "Energy.Active.Import.Interval,Power.Offered,Current.Offered";
    csms.configure(0, "MeterValuesSampledData", measurands);

    csms.input(MINUTE, Input::Tick);
    csms.input(2 * MINUTE, Input::Tick);

    let values: Vec<Vec<String>> = csms
        .sent("MeterValues")
        .iter()
        .map(|sent| {
            let sampled = sent["meterValue"][0]["sampledValue"].as_array().unwrap();
            sampled
                .iter()
                .map(|value| value["value"].as_str().unwrap().to_string())
                .collect()
        })
        .collect();
    assert_eq!(
        values,
        vec![vec!["183", "22080", "32.0"], vec!["183", "22080", "32.0"]]
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
