use crate::station::Station;

pub const DAY_MS: u64 = 86_400_000;
const DEFAULT_PHASES: u32 = 3;

pub struct Schedule {
    pub valid_from: Option<u64>,
    pub valid_to: Option<u64>,
    pub kind: Kind,
    pub start: Option<u64>,
    pub duration_ms: Option<u64>,
    pub unit: Unit,
    pub periods: Vec<Period>,
}

pub enum Kind {
    Absolute,
    Relative,
    Recurring { cycle_ms: Option<u64> },
}

#[derive(Clone, Copy)]
pub enum Unit {
    Watts,
    Amps,
}

pub struct Period {
    pub start_ms: u64,
    pub limit: f64,
    pub phases: Option<u32>,
}

impl Schedule {
    pub fn limit_w(
        &self,
        at: u64,
        transaction_start: Option<u64>,
        station: &Station,
    ) -> Option<f64> {
        if !self.valid_at(at) {
            return None;
        }
        let start = self
            .cycle_start(at, transaction_start)
            .filter(|&start| start <= at)?;
        let elapsed_ms = at - start;
        if self
            .duration_ms
            .is_some_and(|duration| elapsed_ms >= duration)
        {
            return None;
        }
        let period = self
            .periods
            .iter()
            .filter(|period| period.start_ms <= elapsed_ms)
            .max_by_key(|period| period.start_ms)?;
        Some(match self.unit {
            Unit::Watts => period.limit,
            Unit::Amps => {
                let phases = period.phases.unwrap_or(DEFAULT_PHASES).min(station.phases);
                period.limit * station.voltage_v * f64::from(phases)
            }
        })
    }

    pub fn next_change(&self, at: u64, transaction_start: Option<u64>) -> Option<u64> {
        let mut instants = vec![self.valid_from, self.valid_to, self.start];
        if let Some(start) = self.cycle_start(at, transaction_start) {
            let periods = self.periods.iter();
            instants.extend(periods.map(|period| Some(start + period.start_ms)));
            instants.push(self.duration_ms.map(|duration| start + duration));
            instants.push(self.cycle_ms().map(|cycle| start + cycle));
        }
        instants
            .into_iter()
            .flatten()
            .filter(|&instant| instant > at)
            .min()
    }

    fn valid_at(&self, at: u64) -> bool {
        self.valid_from.is_none_or(|from| from <= at) && self.valid_to.is_none_or(|to| at < to)
    }

    fn cycle_start(&self, at: u64, transaction_start: Option<u64>) -> Option<u64> {
        match self.kind {
            Kind::Absolute => Some(self.start.or(transaction_start).unwrap_or(0)),
            Kind::Relative => transaction_start,
            Kind::Recurring { cycle_ms } => {
                let first = self.start?;
                let cycle = cycle_ms?;
                match at.checked_sub(first) {
                    Some(elapsed) => Some(first + elapsed / cycle * cycle),
                    None => Some(first),
                }
            }
        }
    }

    fn cycle_ms(&self) -> Option<u64> {
        match self.kind {
            Kind::Recurring { cycle_ms } => cycle_ms,
            _ => None,
        }
    }
}
