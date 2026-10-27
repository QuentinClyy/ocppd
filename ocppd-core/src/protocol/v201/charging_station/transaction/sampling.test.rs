use crate::api::intent::EvSpec;
use crate::fixtures::v201::metering::{charging, configure, sent, tick, unplug};
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::protocol::v201::event::EventKind;
use serde_json::Value;

fn measurands(meter_value: &Value) -> Vec<String> {
    let mut names: Vec<String> = meter_value["sampledValue"]
        .as_array()
        .unwrap()
        .iter()
        .map(|sampled| sampled["measurand"].as_str().unwrap().to_owned())
        .collect();
    names.dedup();
    names
}

fn register(meter_value: &Value) -> f64 {
    meter_value["sampledValue"][0]["value"].as_f64().unwrap()
}

#[test]
fn given_plugged_ev_when_authorized_then_started_carries_the_started_measurands_per_e01_fr_09_and_tc_j_07()
 {
    let mut run = Run::accepted(86_400);

    charging(&mut run, 0);

    let started = &sent(&run, "TransactionEvent")[0];
    let meter_value = &started["meterValue"][0];
    assert_eq!(
        measurands(meter_value),
        vec!["Energy.Active.Import.Register"]
    );
    assert_eq!(
        meter_value["sampledValue"][0]["context"],
        "Transaction.Begin"
    );
    assert_eq!(
        meter_value["sampledValue"][0]["unitOfMeasure"]["unit"],
        "Wh"
    );
}

#[test]
fn given_booted_station_when_ev_starts_charging_then_periodic_sample_due_after_the_interval_per_j02_fr_10()
 {
    let mut run = Run::accepted(86_400);

    charging(&mut run, 0);

    assert_eq!(run.station.next_deadline(), Some(MINUTE));
}

#[test]
fn given_charging_ev_when_the_sample_interval_passes_then_a_meter_sample_with_power_and_soc() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);

    tick(&mut run, MINUTE);

    let samples: Vec<_> = run
        .events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::MeterSample {
                evse, power_w, soc, ..
            } => Some((*evse, *power_w, *soc)),
            _ => None,
        })
        .collect();
    assert_eq!(samples.len(), 1);
    let (evse, power_w, soc) = samples[0];
    assert_eq!(evse, 1);
    assert!(power_w > 0.0, "{power_w}");
    assert!(soc > EvSpec::default().soc, "{soc}");
}

#[test]
fn given_idle_station_when_the_sample_interval_passes_then_no_meter_sample() {
    let mut run = Run::accepted(86_400);

    tick(&mut run, MINUTE);

    let sampled = run
        .events
        .iter()
        .any(|kind| matches!(kind, EventKind::MeterSample { .. }));
    assert!(!sampled);
}

#[test]
fn given_charging_ev_when_the_sample_interval_passes_then_updated_with_periodic_samples_per_j02_fr_10_and_fr_11_and_tc_j_09()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);

    tick(&mut run, MINUTE);

    let periodic = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(periodic["triggerReason"], "MeterValuePeriodic");
    let meter_value = &periodic["meterValue"][0];
    assert_eq!(
        measurands(meter_value),
        vec![
            "Energy.Active.Import.Register",
            "Power.Active.Import",
            "Current.Import",
            "Current.Offered",
            "Voltage"
        ]
    );
    assert_eq!(meter_value["timestamp"], periodic["timestamp"], "J02.FR.10");
    assert_eq!(meter_value["sampledValue"][0]["context"], "Sample.Periodic");
    assert!(
        meter_value["sampledValue"][1]["value"].as_f64().unwrap() > 0.0,
        "charging power"
    );
}

#[test]
fn given_charging_ev_without_ended_interval_when_unplugged_then_ended_carries_start_and_end_values_per_e06_fr_11_and_tc_j_10()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);

    unplug(&mut run, 10 * MINUTE);

    let ended = sent(&run, "TransactionEvent").pop().unwrap();
    let samples = ended["meterValue"].as_array().unwrap();
    assert_eq!(samples.len(), 2, "TxEndedInterval 0: start and end only");
    assert_eq!(
        samples[0]["sampledValue"][0]["context"],
        "Transaction.Begin"
    );
    assert_eq!(samples[1]["sampledValue"][0]["context"], "Transaction.End");
    assert!(
        register(&samples[1]) > register(&samples[0]),
        "J02.FR.16 monotonic"
    );
}

#[test]
fn given_ended_interval_of_2_min_when_unplugged_after_5_min_then_ended_carries_the_intermediate_samples()
 {
    let mut run = Run::accepted(86_400);
    configure(
        &mut run,
        &[
            ("SampledDataCtrlr.TxEndedInterval", "120"),
            ("SampledDataCtrlr.TxUpdatedInterval", "0"),
        ],
    );
    charging(&mut run, 0);
    for minute in [2, 4] {
        tick(&mut run, minute * MINUTE);
    }

    unplug(&mut run, 5 * MINUTE);

    let ended = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(ended["meterValue"].as_array().unwrap().len(), 4);
}

#[test]
fn given_empty_started_measurands_when_ev_starts_charging_then_started_without_meter_value() {
    let mut run = Run::accepted(86_400);
    configure(&mut run, &[("SampledDataCtrlr.TxStartedMeasurands", "")]);

    charging(&mut run, 0);

    assert!(
        sent(&run, "TransactionEvent")[0]
            .get("meterValue")
            .is_none()
    );
}
