use super::messages::meter_value::{
    Location, Measurand, MeterValue, ReadingContext, SampledValue, UnitOfMeasure,
};
use crate::charge::Sample;
use crate::date_time::DateTime;
use crate::station::Station;

pub fn meter_value(now: u64, sample: &Sample, station: &Station) -> MeterValue {
    let current_a = sample.power_w / (station.voltage_v * f64::from(station.phases));
    MeterValue {
        timestamp: DateTime { unix_ms: now },
        sampled_value: vec![
            sampled(
                format!("{:.0}", sample.register_wh),
                Measurand::EnergyActiveImportRegister,
                UnitOfMeasure::Wh,
            ),
            sampled(
                format!("{:.0}", sample.power_w),
                Measurand::PowerActiveImport,
                UnitOfMeasure::W,
            ),
            sampled(
                format!("{current_a:.1}"),
                Measurand::CurrentImport,
                UnitOfMeasure::A,
            ),
            sampled(
                format!("{:.1}", station.voltage_v),
                Measurand::Voltage,
                UnitOfMeasure::V,
            ),
        ],
    }
}

fn sampled(value: String, measurand: Measurand, unit: UnitOfMeasure) -> SampledValue {
    SampledValue {
        value,
        context: Some(ReadingContext::SamplePeriodic),
        format: None,
        measurand: Some(measurand),
        phase: None,
        location: Some(Location::Outlet),
        unit: Some(unit),
    }
}

#[cfg(test)]
#[path = "meter_values.test.rs"]
mod tests;
