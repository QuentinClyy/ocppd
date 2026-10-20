const KNEE_SOC: f64 = 0.8;
const CUTOFF: f64 = 0.02;
const MS_PER_HOUR: f64 = 3_600_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ev {
    pub capacity_wh: f64,
    pub max_w: f64,
    pub target_soc: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charging {
    pub start: u64,
    pub start_soc: f64,
    pub start_register_wh: f64,
    pub limit_w: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub register_wh: f64,
    pub power_w: f64,
    pub soc: f64,
}

pub fn sample(charging: &Charging, ev: &Ev, now: u64) -> Sample {
    let power = charging.limit_w.min(ev.max_w);
    let end_soc = end_soc(ev);
    let hours = now.saturating_sub(charging.start) as f64 / MS_PER_HOUR;
    let soc = if power > 0.0 {
        soc_after(charging.start_soc, power, ev.capacity_wh, hours).min(end_soc)
    } else {
        charging.start_soc
    };
    let soc = soc.max(charging.start_soc);
    let power_w = if soc >= end_soc || power <= 0.0 {
        0.0
    } else if soc < KNEE_SOC {
        power
    } else {
        power * (1.0 - soc) / (1.0 - KNEE_SOC)
    };
    Sample {
        register_wh: charging.start_register_wh + (soc - charging.start_soc) * ev.capacity_wh,
        power_w,
        soc,
    }
}

pub fn finish_time(charging: &Charging, ev: &Ev) -> Option<u64> {
    let power = charging.limit_w.min(ev.max_w);
    let end_soc = end_soc(ev);
    if charging.start_soc >= end_soc {
        return Some(charging.start);
    }
    if power <= 0.0 {
        return None;
    }
    let hours = hours_to(charging.start_soc, end_soc, power, ev.capacity_wh);
    Some(charging.start + (hours * MS_PER_HOUR).ceil() as u64)
}

pub fn reanchor(charging: &Charging, ev: &Ev, now: u64, limit_w: f64) -> Charging {
    let at_now = sample(charging, ev, now);
    Charging {
        start: now,
        start_soc: at_now.soc,
        start_register_wh: at_now.register_wh,
        limit_w,
    }
}

fn end_soc(ev: &Ev) -> f64 {
    let cutoff_soc = 1.0 - CUTOFF * (1.0 - KNEE_SOC);
    ev.target_soc.min(cutoff_soc)
}

fn soc_after(start_soc: f64, power: f64, capacity_wh: f64, hours: f64) -> f64 {
    let hours_to_knee = ((KNEE_SOC - start_soc) * capacity_wh / power).max(0.0);
    if hours <= hours_to_knee {
        return start_soc + power * hours / capacity_wh;
    }
    let taper_start = start_soc.max(KNEE_SOC);
    let taper_hours = hours - hours_to_knee;
    1.0 - (1.0 - taper_start) * (-power * taper_hours / ((1.0 - KNEE_SOC) * capacity_wh)).exp()
}

fn hours_to(start_soc: f64, end_soc: f64, power: f64, capacity_wh: f64) -> f64 {
    let linear_end = end_soc.min(KNEE_SOC);
    let linear = ((linear_end - start_soc) * capacity_wh / power).max(0.0);
    if end_soc <= KNEE_SOC {
        return linear;
    }
    let taper_start = start_soc.max(KNEE_SOC);
    let taper =
        ((1.0 - taper_start) / (1.0 - end_soc)).ln() * (1.0 - KNEE_SOC) * capacity_wh / power;
    linear + taper
}

#[cfg(test)]
#[path = "charge.test.rs"]
mod tests;
