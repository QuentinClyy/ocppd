use super::Station;
use super::charge::{self, Current, DC_EFFICIENCY, Sample};
use super::jitter;

const VOLTAGE_NOISE: f64 = 0.005;
const CURRENT_NOISE: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Outlet,
    Inlet,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub side: Side,
    pub phase: Option<usize>,
    pub voltage_v: f64,
    pub current_a: f64,
}

pub fn lines(station: &Station, index: usize, sample: &Sample, now: u64) -> Vec<Line> {
    let exact = match station.connectors[index].current {
        Current::Ac => ac_lines(station, index, sample),
        Current::Dc => dc_lines(station, sample),
    };
    exact
        .into_iter()
        .enumerate()
        .map(|(position, line)| Line {
            voltage_v: noisy(station, now, 2 * position, VOLTAGE_NOISE, line.voltage_v),
            current_a: noisy(
                station,
                now,
                2 * position + 1,
                CURRENT_NOISE,
                line.current_a,
            ),
            ..line
        })
        .collect()
}

pub fn power_w(lines: &[Line], side: Side) -> Option<f64> {
    let mut on_side = lines.iter().filter(|line| line.side == side).peekable();
    on_side.peek()?;
    Some(on_side.map(|line| line.voltage_v * line.current_a).sum())
}

pub fn offered_a(station: &Station, index: usize, sample: &Sample, offered_w: f64) -> f64 {
    match station.connectors[index].current {
        Current::Ac => offered_w / (station.voltage_v * f64::from(used_phases(station, index))),
        Current::Dc if sample.voltage_v > 0.0 => {
            (offered_w * DC_EFFICIENCY / sample.voltage_v).min(station.max_output_a)
        }
        Current::Dc => station.max_output_a,
    }
}

fn used_phases(station: &Station, index: usize) -> u32 {
    let supply = station.supply(index);
    match &station.connectors[index].plugged {
        Some(plugged) => charge::ac_phases(&supply, &plugged.ev),
        None => supply.phases,
    }
}

fn ac_lines(station: &Station, index: usize, sample: &Sample) -> Vec<Line> {
    let used = used_phases(station, index);
    let amps = sample.power_w / (station.voltage_v * f64::from(used));
    (0..station.phases)
        .map(|phase| Line {
            side: Side::Outlet,
            phase: Some(phase as usize),
            voltage_v: station.voltage_v,
            current_a: if phase < used { amps } else { 0.0 },
        })
        .collect()
}

fn dc_lines(station: &Station, sample: &Sample) -> Vec<Line> {
    let inlet_a = sample.power_w / DC_EFFICIENCY / (station.voltage_v * f64::from(station.phases));
    let outlet = Line {
        side: Side::Outlet,
        phase: None,
        voltage_v: sample.voltage_v,
        current_a: sample.current_a,
    };
    let inlet = (0..station.phases).map(|phase| Line {
        side: Side::Inlet,
        phase: Some(phase as usize),
        voltage_v: station.voltage_v,
        current_a: inlet_a,
    });
    std::iter::once(outlet).chain(inlet).collect()
}

fn noisy(station: &Station, now: u64, quantity: usize, amplitude: f64, value: f64) -> f64 {
    if !station.jitter {
        return value;
    }
    let salt = now.wrapping_mul(64).wrapping_add(quantity as u64);
    value * (1.0 + amplitude * (2.0 * jitter::fraction(&station.id, salt) - 1.0))
}

#[cfg(test)]
#[path = "electrical.test.rs"]
mod tests;
