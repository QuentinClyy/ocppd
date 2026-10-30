use serde::Deserialize;

const MS_PER_HOUR: f64 = 3_600_000.0;

type Curve = &'static [(f64, f64)];

const AC_CURVE: Curve = &[(0.0, 1.0), (0.99, 1.0), (1.0, 0.1)];
const DC_CURVE: Curve = &[(0.0, 1.0), (0.5, 1.0), (0.8, 0.5), (1.0, 0.05)];

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Current {
    #[default]
    Ac,
    Dc,
}

impl Current {
    fn curve(self) -> Curve {
        match self {
            Current::Ac => AC_CURVE,
            Current::Dc => DC_CURVE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ev {
    pub capacity_wh: f64,
    pub max_w: f64,
    pub target_soc: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charging {
    pub current: Current,
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
    let curve = charging.current.curve();
    let power = charging.limit_w.min(ev.max_w);
    let end_soc = ev.target_soc.min(1.0);
    let hours = now.saturating_sub(charging.start) as f64 / MS_PER_HOUR;
    let soc = if power > 0.0 {
        soc_after(curve, charging.start_soc, power, ev.capacity_wh, hours).min(end_soc)
    } else {
        charging.start_soc
    };
    let soc = soc.max(charging.start_soc);
    let power_w = if soc >= end_soc || power <= 0.0 {
        0.0
    } else {
        power * fraction_at(curve, soc)
    };
    Sample {
        register_wh: charging.start_register_wh + (soc - charging.start_soc) * ev.capacity_wh,
        power_w,
        soc,
    }
}

pub fn finish_time(charging: &Charging, ev: &Ev) -> Option<u64> {
    let power = charging.limit_w.min(ev.max_w);
    let end_soc = ev.target_soc.min(1.0);
    if charging.start_soc >= end_soc {
        return Some(charging.start);
    }
    if power <= 0.0 {
        return None;
    }
    let curve = charging.current.curve();
    let hours = hours_to(curve, charging.start_soc, end_soc, power, ev.capacity_wh);
    Some(charging.start + (hours * MS_PER_HOUR).ceil() as u64)
}

pub fn reanchor(charging: &Charging, ev: &Ev, now: u64, limit_w: f64) -> Charging {
    let at_now = sample(charging, ev, now);
    Charging {
        start: now,
        start_soc: at_now.soc,
        start_register_wh: at_now.register_wh,
        limit_w,
        ..*charging
    }
}

fn slope(segment: &[(f64, f64)]) -> f64 {
    (segment[1].1 - segment[0].1) / (segment[1].0 - segment[0].0)
}

fn fraction_in(segment: &[(f64, f64)], soc: f64) -> f64 {
    segment[0].1 + slope(segment) * (soc - segment[0].0)
}

fn fraction_at(curve: Curve, soc: f64) -> f64 {
    let segment = curve
        .windows(2)
        .find(|segment| soc < segment[1].0)
        .unwrap_or(&curve[curve.len() - 2..]);
    fraction_in(segment, soc)
}

fn segment_hours(segment: &[(f64, f64)], from: f64, to: f64, power: f64, capacity_wh: f64) -> f64 {
    let (start, end) = (fraction_in(segment, from), fraction_in(segment, to));
    if start == end {
        return (to - from) * capacity_wh / (power * start);
    }
    capacity_wh / (power * slope(segment)) * (end / start).ln()
}

fn segment_soc(segment: &[(f64, f64)], from: f64, power: f64, capacity_wh: f64, hours: f64) -> f64 {
    let start = fraction_in(segment, from);
    if segment[0].1 == segment[1].1 {
        return from + power * start * hours / capacity_wh;
    }
    let k = slope(segment);
    let end = start * (k * power * hours / capacity_wh).exp();
    from + (end - start) / k
}

fn soc_after(curve: Curve, start_soc: f64, power: f64, capacity_wh: f64, hours: f64) -> f64 {
    let mut soc = start_soc;
    let mut left = hours;
    for segment in curve.windows(2).filter(|segment| start_soc < segment[1].0) {
        let needed = segment_hours(segment, soc, segment[1].0, power, capacity_wh);
        if left <= needed {
            return segment_soc(segment, soc, power, capacity_wh, left);
        }
        left -= needed;
        soc = segment[1].0;
    }
    soc
}

fn hours_to(curve: Curve, start_soc: f64, end_soc: f64, power: f64, capacity_wh: f64) -> f64 {
    curve
        .windows(2)
        .map(|segment| {
            (
                start_soc.max(segment[0].0),
                end_soc.min(segment[1].0),
                segment,
            )
        })
        .filter(|(from, to, _)| from < to)
        .map(|(from, to, segment)| segment_hours(segment, from, to, power, capacity_wh))
        .sum()
}

#[cfg(test)]
#[path = "charge.test.rs"]
mod tests;
