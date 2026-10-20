use super::messages::meter_value::{
    Location, Measurand, MeterValue, ReadingContext, SampledValue, UnitOfMeasure,
};
use crate::charge::Sample;
use crate::date_time::DateTime;
use crate::station::Station;

pub fn meter_value(
    now: u64,
    sample: &Sample,
    station: &Station,
    measurands: &[Measurand],
) -> MeterValue {
    MeterValue {
        timestamp: DateTime { unix_ms: now },
        sampled_value: measurands
            .iter()
            .filter_map(|&measurand| sampled(measurand, sample, station))
            .collect(),
    }
}

fn sampled(measurand: Measurand, sample: &Sample, station: &Station) -> Option<SampledValue> {
    let current_a = sample.power_w / (station.voltage_v * f64::from(station.phases));
    let (value, unit) = match measurand {
        Measurand::EnergyActiveImportRegister => {
            (format!("{:.0}", sample.register_wh), UnitOfMeasure::Wh)
        }
        Measurand::PowerActiveImport => (format!("{:.0}", sample.power_w), UnitOfMeasure::W),
        Measurand::CurrentImport => (format!("{current_a:.1}"), UnitOfMeasure::A),
        Measurand::Voltage => (format!("{:.1}", station.voltage_v), UnitOfMeasure::V),
        _ => return None,
    };
    Some(SampledValue {
        value,
        context: Some(ReadingContext::SamplePeriodic),
        format: None,
        measurand: Some(measurand),
        phase: None,
        location: Some(Location::Outlet),
        unit: Some(unit),
    })
}

#[cfg(test)]
#[path = "meter_values.test.rs"]
mod tests;
