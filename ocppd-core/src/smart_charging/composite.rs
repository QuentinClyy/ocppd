use super::schedule::Unit;
use crate::station::Station;

pub struct Sampled {
    pub start_s: u32,
    pub limit: f64,
}

pub fn periods(
    station: &Station,
    unit: Unit,
    from: u64,
    end: u64,
    limit_w: impl Fn(u64) -> f64,
    next_change: impl Fn(u64) -> Option<u64>,
) -> Vec<Sampled> {
    let mut periods: Vec<Sampled> = Vec::new();
    let mut at = from;
    while at < end {
        let limit = in_unit(station, limit_w(at), unit);
        if periods.last().is_none_or(|last| last.limit != limit) {
            periods.push(Sampled {
                start_s: ((at - from) / 1000) as u32,
                limit,
            });
        }
        at = next_change(at).unwrap_or(end);
    }
    periods
}

fn in_unit(station: &Station, watts: f64, unit: Unit) -> f64 {
    match unit {
        Unit::Watts => watts.round(),
        Unit::Amps => tenths_down(watts / (station.voltage_v * f64::from(station.phases))),
    }
}

fn tenths_down(rate: f64) -> f64 {
    (rate * 10.0 + 1e-9).floor() / 10.0
}
