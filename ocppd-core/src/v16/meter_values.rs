use super::messages::meter_value::{
    Location, Measurand, MeterValue, Phase, ReadingContext, SampledValue, UnitOfMeasure,
};
use crate::charge::Sample;
use crate::date_time::DateTime;
use crate::station::Station;

const LINES: [(Phase, Phase); 3] = [
    (Phase::L1, Phase::L1N),
    (Phase::L2, Phase::L2N),
    (Phase::L3, Phase::L3N),
];

pub struct Reading {
    pub sample: Sample,
    pub offered_w: f64,
    pub interval_wh: f64,
}

pub fn meter_value(
    now: u64,
    reading: &Reading,
    station: &Station,
    measurands: &[Measurand],
) -> MeterValue {
    MeterValue {
        timestamp: DateTime { unix_ms: now },
        sampled_value: measurands
            .iter()
            .flat_map(|&measurand| sampled(measurand, reading, station))
            .collect(),
    }
}

fn sampled(measurand: Measurand, reading: &Reading, station: &Station) -> Vec<SampledValue> {
    let phases = &LINES[..(station.phases as usize).min(LINES.len())];
    let per_phase = |watts: f64| watts / (station.voltage_v * f64::from(station.phases));
    let total = |value: String, unit| vec![value_of(measurand, value, None, unit)];
    match measurand {
        Measurand::EnergyActiveImportRegister => total(
            format!("{:.0}", reading.sample.register_wh),
            UnitOfMeasure::Wh,
        ),
        Measurand::EnergyActiveImportInterval => {
            total(format!("{:.0}", reading.interval_wh), UnitOfMeasure::Wh)
        }
        Measurand::PowerActiveImport => {
            total(format!("{:.0}", reading.sample.power_w), UnitOfMeasure::W)
        }
        Measurand::PowerOffered => total(format!("{:.0}", reading.offered_w), UnitOfMeasure::W),
        Measurand::CurrentOffered => total(
            format!("{:.1}", per_phase(reading.offered_w)),
            UnitOfMeasure::A,
        ),
        Measurand::CurrentImport => phases
            .iter()
            .map(|&(line, _)| {
                let amps = format!("{:.1}", per_phase(reading.sample.power_w));
                value_of(measurand, amps, Some(line), UnitOfMeasure::A)
            })
            .collect(),
        Measurand::Voltage => phases
            .iter()
            .map(|&(_, to_neutral)| {
                let volts = format!("{:.1}", station.voltage_v);
                value_of(measurand, volts, Some(to_neutral), UnitOfMeasure::V)
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn value_of(
    measurand: Measurand,
    value: String,
    phase: Option<Phase>,
    unit: UnitOfMeasure,
) -> SampledValue {
    SampledValue {
        value,
        context: Some(ReadingContext::SamplePeriodic),
        format: None,
        measurand: Some(measurand),
        phase,
        location: Some(Location::Outlet),
        unit: Some(unit),
    }
}

#[cfg(test)]
#[path = "meter_values.test.rs"]
mod tests;
