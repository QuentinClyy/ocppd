use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

fn charging(run: &mut Run, now: u64) {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(now);
    let intent = Intent::Authorize {
        connector: None,
        id_tag: "04A2B3C4".into(),
    };
    run.input(now, Input::Intent(intent));
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(now);
}

fn sent(run: &Run, action: &str) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, sent, _)| sent == action)
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

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

fn tick(run: &mut Run, now: u64) {
    run.input(now, Input::Tick);
    run.answer_all(now);
}

fn configure(run: &mut Run, settings: &[(&str, &str)]) {
    for (key, value) in settings {
        assert!(run.station.configure(key, value), "{key}");
    }
}

fn register(meter_value: &Value) -> f64 {
    meter_value["sampledValue"][0]["value"].as_f64().unwrap()
}

fn unplug(run: &mut Run, now: u64) {
    run.input(now, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(now);
}

fn clock_aligned_two_evses() -> Run {
    let mut run = Run::accepted_with(2, 86_400);
    configure(
        &mut run,
        &[
            ("AlignedDataCtrlr.Interval", "900"),
            ("SampledDataCtrlr.TxUpdatedInterval", "0"),
        ],
    );
    run
}

fn aligned_ended() -> Run {
    let mut run = Run::accepted(86_400);
    configure(
        &mut run,
        &[
            ("AlignedDataCtrlr.TxEndedInterval", "60"),
            ("SampledDataCtrlr.TxEndedMeasurands", ""),
            ("SampledDataCtrlr.TxUpdatedInterval", "0"),
        ],
    );
    run
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
fn given_aligned_interval_of_15_min_when_time_passes_then_next_boundary_scheduled_per_j01_fr_14() {
    let mut run = clock_aligned_two_evses();

    run.input(100_000, Input::Tick);

    assert_eq!(run.station.next_deadline(), Some(900_000));
}

#[test]
fn given_one_charging_and_one_idle_evse_when_the_clock_boundary_passes_then_meter_values_for_the_idle_one_and_a_clock_event_for_the_other_per_j01_fr_02_fr_14_fr_22_and_tc_j_01_j_02()
 {
    let mut run = clock_aligned_two_evses();
    run.input(100_000, Input::Tick);
    charging(&mut run, 200_000);

    tick(&mut run, 900_000);

    let meter_values = sent(&run, "MeterValues");
    assert_eq!(meter_values.len(), 1, "only the idle EVSE");
    assert_eq!(meter_values[0]["evseId"], 2, "J01.FR.02");
    assert_eq!(
        meter_values[0]["meterValue"][0]["sampledValue"][0]["context"],
        "Sample.Clock"
    );
    let clock = sent(&run, "TransactionEvent").pop().unwrap();
    assert_eq!(clock["triggerReason"], "MeterValueClock");
    assert_eq!(clock["evse"]["id"], 1);
    assert_eq!(run.station.next_deadline(), Some(1_800_000));
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

#[test]
fn given_booted_station_when_unsupported_measurand_configured_then_refused() {
    let mut run = Run::accepted(86_400);

    let accepted = run
        .station
        .configure("SampledDataCtrlr.TxUpdatedMeasurands", "Frequency");

    assert!(!accepted);
}

#[test]
fn given_booted_station_when_supported_measurands_configured_then_accepted() {
    let mut run = Run::accepted(86_400);

    let accepted = run.station.configure(
        "SampledDataCtrlr.TxUpdatedMeasurands",
        "Voltage,Current.Import",
    );

    assert!(accepted);
}

#[test]
fn given_aligned_ended_interval_of_1_min_when_ev_starts_charging_then_first_boundary_scheduled_per_j01()
 {
    let mut run = aligned_ended();

    charging(&mut run, 10_000);

    assert_eq!(run.station.next_deadline(), Some(MINUTE));
}

#[test]
fn given_aligned_ended_interval_and_two_boundaries_passed_when_unplugged_then_ended_carries_the_clock_values_per_j01_and_tc_j_03()
 {
    let mut run = aligned_ended();
    charging(&mut run, 10_000);
    for now in [MINUTE, 2 * MINUTE] {
        tick(&mut run, now);
    }

    unplug(&mut run, 2 * MINUTE + 5_000);

    let ended = sent(&run, "TransactionEvent").pop().unwrap();
    let samples: Vec<_> = ended["meterValue"]
        .as_array()
        .unwrap()
        .iter()
        .map(|meter_value| {
            (
                meter_value["timestamp"].clone(),
                meter_value["sampledValue"][0]["context"].clone(),
            )
        })
        .collect();
    assert_eq!(
        samples,
        vec![
            (json!("1970-01-01T00:01:00Z"), json!("Sample.Clock")),
            (json!("1970-01-01T00:02:00Z"), json!("Sample.Clock")),
        ]
    );
}
