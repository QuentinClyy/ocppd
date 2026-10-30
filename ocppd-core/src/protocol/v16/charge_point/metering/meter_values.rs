use crate::protocol::v16::messages::datatypes::meter_value::{
    Location, Measurand, MeterValue, Phase, ReadingContext, SampledValue, UnitOfMeasure,
};
use crate::station::charge::Sample;
use crate::station::electrical::{self, Line, Side};
use crate::wire::date_time::DateTime;

const LINES: [(Phase, Phase); 3] = [
    (Phase::L1, Phase::L1N),
    (Phase::L2, Phase::L2N),
    (Phase::L3, Phase::L3N),
];

pub struct Reading {
    pub sample: Sample,
    pub lines: Vec<Line>,
    pub offered_w: f64,
    pub offered_a: f64,
    pub soc: Option<f64>,
    pub interval_wh: f64,
    pub context: ReadingContext,
}

pub fn meter_value(now: u64, reading: &Reading, measurands: &[Measurand]) -> MeterValue {
    MeterValue {
        timestamp: DateTime { unix_ms: now },
        sampled_value: measurands
            .iter()
            .flat_map(|&measurand| sampled(measurand, reading))
            .collect(),
    }
}

fn sampled(measurand: Measurand, reading: &Reading) -> Vec<SampledValue> {
    let value = |value: String, phase, side, unit| SampledValue {
        value,
        context: Some(reading.context),
        format: None,
        measurand: Some(measurand),
        phase,
        location: Some(location(side)),
        unit: Some(unit),
    };
    let total = |text: String, unit| vec![value(text, None, Side::Outlet, unit)];
    let lines = reading.lines.iter();
    match measurand {
        Measurand::EnergyActiveImportRegister => total(
            format!("{:.0}", reading.sample.register_wh),
            UnitOfMeasure::Wh,
        ),
        Measurand::EnergyActiveImportInterval => {
            total(format!("{:.0}", reading.interval_wh), UnitOfMeasure::Wh)
        }
        Measurand::PowerActiveImport => [Side::Outlet, Side::Inlet]
            .into_iter()
            .filter_map(|side| {
                let watts = electrical::power_w(&reading.lines, side)?;
                Some(value(format!("{watts:.0}"), None, side, UnitOfMeasure::W))
            })
            .collect(),
        Measurand::PowerOffered => total(format!("{:.0}", reading.offered_w), UnitOfMeasure::W),
        Measurand::CurrentOffered => total(format!("{:.1}", reading.offered_a), UnitOfMeasure::A),
        Measurand::CurrentImport => lines
            .map(|line| {
                let phase = line.phase.map(|phase| LINES[phase].0);
                let amps = format!("{:.1}", line.current_a);
                value(amps, phase, line.side, UnitOfMeasure::A)
            })
            .collect(),
        Measurand::Voltage => lines
            .map(|line| {
                let phase = line.phase.map(|phase| LINES[phase].1);
                let volts = format!("{:.1}", line.voltage_v);
                value(volts, phase, line.side, UnitOfMeasure::V)
            })
            .collect(),
        Measurand::SoC => reading
            .soc
            .map(|soc| {
                value(
                    format!("{:.1}", soc * 100.0),
                    None,
                    Side::Outlet,
                    UnitOfMeasure::Percent,
                )
            })
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn location(side: Side) -> Location {
    match side {
        Side::Outlet => Location::Outlet,
        Side::Inlet => Location::Inlet,
    }
}

#[cfg(test)]
#[path = "meter_values.test.rs"]
mod tests;
