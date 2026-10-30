use serde::Deserialize;

const MS_PER_HOUR: f64 = 3_600_000.0;
const SOC_STEP: f64 = 0.001;
const EMPTY_VOLTAGE: f64 = 0.9;
const VOLTAGE_RISE: f64 = 0.25;
const DERATE: f64 = 0.25;
const PEAK_SOC: f64 = 0.2;
const CONSTANT_VOLTAGE_SOC: f64 = 0.6;
const TERMINATION_C_RATE: f64 = 0.05;
const STARTUP_MS: u64 = 5_000;
const RAMP_A_PER_S: f64 = 50.0;
pub const DC_EFFICIENCY: f64 = 0.95;

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Current {
    #[default]
    Ac,
    Dc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ev {
    pub capacity_wh: f64,
    pub pack_v: f64,
    pub ac_w: f64,
    pub ac_phases: u32,
    pub dc_w: f64,
    pub target_soc: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Supply {
    pub current: Current,
    pub phases: u32,
    pub max_output_a: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charging {
    pub supply: Supply,
    pub start: u64,
    pub start_soc: f64,
    pub start_register_wh: f64,
    pub limit_w: f64,
    pub ramp: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub register_wh: f64,
    pub power_w: f64,
    pub current_a: f64,
    pub voltage_v: f64,
    pub soc: f64,
}

pub fn voltage(ev: &Ev, soc: f64) -> f64 {
    ev.pack_v * (EMPTY_VOLTAGE + VOLTAGE_RISE * soc)
}

pub fn ac_phases(supply: &Supply, ev: &Ev) -> u32 {
    supply.phases.min(ev.ac_phases).max(1)
}

pub fn sample(charging: &Charging, ev: &Ev, now: u64) -> Sample {
    let soc = soc_at(charging, ev, now);
    let amps = if soc >= end_soc(ev) {
        0.0
    } else {
        amps(charging, ev, soc).min(ramp_cap(charging, now))
    };
    let volts = voltage(ev, soc);
    Sample {
        register_wh: charging.start_register_wh + energy_wh(ev, charging.start_soc, soc),
        power_w: volts * amps,
        current_a: amps,
        voltage_v: volts,
        soc,
    }
}

pub fn finish_time(charging: &Charging, ev: &Ev) -> Option<u64> {
    let end = end_soc(ev);
    if charging.start_soc >= end {
        return Some(charging.start);
    }
    if charging.limit_w <= 0.0 {
        return None;
    }
    let hours = hours_between(charging, ev, charging.start_soc, end);
    hours
        .is_finite()
        .then(|| charging.start + startup_ms(charging, ev) + (hours * MS_PER_HOUR).ceil() as u64)
}

pub fn reanchor(charging: &Charging, ev: &Ev, now: u64, limit_w: f64) -> Charging {
    let at_now = sample(charging, ev, now);
    Charging {
        start: now,
        start_soc: at_now.soc,
        start_register_wh: at_now.register_wh,
        limit_w,
        ramp: false,
        ..*charging
    }
}

fn end_soc(ev: &Ev) -> f64 {
    ev.target_soc.min(1.0)
}

fn capacity_ah(ev: &Ev) -> f64 {
    ev.capacity_wh / (ev.pack_v * (EMPTY_VOLTAGE + VOLTAGE_RISE / 2.0))
}

fn energy_wh(ev: &Ev, from: f64, to: f64) -> f64 {
    let integral = |soc: f64| EMPTY_VOLTAGE * soc + VOLTAGE_RISE / 2.0 * soc * soc;
    capacity_ah(ev) * ev.pack_v * (integral(to) - integral(from))
}

fn accepted_a(ev: &Ev, soc: f64) -> f64 {
    let peak = ev.dc_w / (voltage(ev, PEAK_SOC) * (1.0 - DERATE * PEAK_SOC));
    let termination = capacity_ah(ev) * TERMINATION_C_RATE;
    let derated = peak * (1.0 - DERATE * soc);
    let constant_voltage = peak * (1.0 - soc) / (1.0 - CONSTANT_VOLTAGE_SOC) + termination;
    derated.min(constant_voltage)
}

fn supplied_a(charging: &Charging, ev: &Ev, soc: f64) -> f64 {
    let volts = voltage(ev, soc);
    let supply = &charging.supply;
    match supply.current {
        Current::Ac => {
            let share = f64::from(ac_phases(supply, ev)) / f64::from(supply.phases);
            (charging.limit_w * share).min(ev.ac_w) / volts
        }
        Current::Dc => (charging.limit_w * DC_EFFICIENCY / volts).min(supply.max_output_a),
    }
}

fn amps(charging: &Charging, ev: &Ev, soc: f64) -> f64 {
    supplied_a(charging, ev, soc)
        .min(accepted_a(ev, soc))
        .max(0.0)
}

fn ramps(charging: &Charging) -> bool {
    charging.ramp && charging.supply.current == Current::Dc
}

fn ramp_cap(charging: &Charging, now: u64) -> f64 {
    if !ramps(charging) {
        return f64::INFINITY;
    }
    let ramping_ms = now.saturating_sub(charging.start + STARTUP_MS);
    RAMP_A_PER_S * ramping_ms as f64 / 1000.0
}

fn startup_ms(charging: &Charging, ev: &Ev) -> u64 {
    if !ramps(charging) {
        return 0;
    }
    let ramp_ms = amps(charging, ev, charging.start_soc) / RAMP_A_PER_S * 1000.0;
    STARTUP_MS + (ramp_ms / 2.0) as u64
}

fn soc_at(charging: &Charging, ev: &Ev, now: u64) -> f64 {
    let charged_ms = now.saturating_sub(charging.start + startup_ms(charging, ev));
    let hours = charged_ms as f64 / MS_PER_HOUR;
    soc_after(charging, ev, hours)
        .min(end_soc(ev))
        .max(charging.start_soc)
}

fn soc_per_hour(charging: &Charging, ev: &Ev, soc: f64) -> f64 {
    amps(charging, ev, soc) / capacity_ah(ev)
}

fn cells(from: f64, to: f64) -> impl Iterator<Item = (f64, f64)> {
    let first = (from / SOC_STEP).floor() as usize;
    let last = (to / SOC_STEP).ceil() as usize;
    (first..last).map(move |cell| {
        let low = (cell as f64 * SOC_STEP).max(from);
        let high = ((cell + 1) as f64 * SOC_STEP).min(to);
        (low, high)
    })
}

fn hours_between(charging: &Charging, ev: &Ev, from: f64, to: f64) -> f64 {
    cells(from, to)
        .map(|(low, high)| (high - low) / soc_per_hour(charging, ev, (low + high) / 2.0))
        .sum()
}

fn soc_after(charging: &Charging, ev: &Ev, hours: f64) -> f64 {
    let mut left = hours;
    for (low, high) in cells(charging.start_soc, 1.0) {
        let rate = soc_per_hour(charging, ev, (low + high) / 2.0);
        if rate <= 0.0 {
            return low;
        }
        let needed = (high - low) / rate;
        if left <= needed {
            return low + left * rate;
        }
        left -= needed;
    }
    1.0
}

#[cfg(test)]
#[path = "charge.test.rs"]
mod tests;
