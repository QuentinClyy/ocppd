use crate::api::io::Input;
use crate::fixtures::v201::metering::{charging, configure, sent, tick, unplug};
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::protocol::v201::messages::datatypes::meter_value::ReadingContext;
use crate::station::charge::Current;
use serde_json::{Value, json};

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

#[test]
fn given_dc_station_charging_when_a_meter_value_is_sampled_then_dc_voltage_at_the_outlet_and_grid_phases_at_the_inlet()
 {
    let mut run = Run::accepted(86_400);
    run.station.station.connectors[0].current = Current::Dc;
    configure(
        &mut run,
        &[("SampledDataCtrlr.TxUpdatedMeasurands", "Voltage")],
    );
    charging(&mut run, 100_000);

    let sampled = run.station.meter_value(
        200_000,
        0,
        ("SampledDataCtrlr", "TxUpdatedMeasurands"),
        ReadingContext::SamplePeriodic,
    );

    let values = json!(sampled.unwrap())["sampledValue"].clone();
    let at = |location: &str| -> Vec<Value> {
        let all = values.as_array().unwrap().iter();
        all.filter(|value| value["location"] == location)
            .cloned()
            .collect()
    };
    let outlet = at("Outlet");
    assert_eq!(outlet.len(), 1);
    assert!(outlet[0].get("phase").is_none());
    assert!(outlet[0]["value"].as_f64().unwrap() > 300.0);
    let inlet: Vec<Value> = at("Inlet")
        .iter()
        .map(|value| value["phase"].clone())
        .collect();
    assert_eq!(inlet, vec![json!("L1-N"), json!("L2-N"), json!("L3-N")]);
}
