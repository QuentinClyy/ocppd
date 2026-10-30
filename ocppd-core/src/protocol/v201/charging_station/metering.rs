use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::messages::datatypes::meter_value::{
    Location, Measurand, MeterValue, Phase, ReadingContext, SampledValue, UnitOfMeasure,
};
use crate::protocol::v201::messages::meter_values::MeterValuesRequest;
use crate::protocol::v201::messages::transaction::{TransactionEventType, TriggerReason};
use crate::station::charge::Current;
use crate::station::electrical::{self, Side};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::{Value, json};

pub const SAMPLED_DATA_CTRLR: &str = "SampledDataCtrlr";
pub const ALIGNED_DATA_CTRLR: &str = "AlignedDataCtrlr";
pub const TX_ENDED_MEASURANDS: &str = "TxEndedMeasurands";
pub const INTERVAL: &str = "Interval";
pub const MEASURANDS_VARIABLE: &str = "Measurands";
const AC_MEASURANDS: &str = "Energy.Active.Import.Register,Power.Active.Import,Power.Offered,Current.Import,Current.Offered,Voltage";
const DC_MEASURANDS: &str = "Energy.Active.Import.Register,Power.Active.Import,Power.Offered,Current.Import,Current.Offered,Voltage,SoC";

pub fn measurands(current: Current) -> &'static str {
    match current {
        Current::Ac => AC_MEASURANDS,
        Current::Dc => DC_MEASURANDS,
    }
}

const LINES: [(Phase, Phase); 3] = [
    (Phase::L1, Phase::L1N),
    (Phase::L2, Phase::L2N),
    (Phase::L3, Phase::L3N),
];

impl ChargingStation {
    pub(super) fn aligned_tick(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(interval) = self.interval_ms(ALIGNED_DATA_CTRLR, INTERVAL) else {
            self.next_aligned = None;
            return;
        };
        let next = *self
            .next_aligned
            .get_or_insert(now - now % interval + interval);
        if now < next {
            return;
        }
        self.next_aligned = Some(now - now % interval + interval);
        for index in 0..self.station.connectors.len() {
            let measurands = (ALIGNED_DATA_CTRLR, MEASURANDS_VARIABLE);
            let Some(sample) =
                self.meter_value(now, index, measurands, ReadingContext::SampleClock)
            else {
                continue;
            };
            let evse_id = index as u32 + 1;
            match self.sessions[index].transaction {
                Some(_) => {
                    let updated = TransactionEventType::Updated;
                    let trigger = TriggerReason::MeterValueClock;
                    self.send_transaction_event(out, now, index, updated, trigger, vec![sample]);
                }
                None => {
                    let request = MeterValuesRequest {
                        evse_id,
                        meter_value: vec![sample],
                    };
                    self.outbox.push("MeterValues", evse_id, json!(request));
                }
            }
        }
    }

    pub(super) fn meter_value(
        &self,
        now: u64,
        index: usize,
        measurands: (&str, &str),
        context: ReadingContext,
    ) -> Option<MeterValue> {
        let (component, variable) = measurands;
        let list = self
            .device_model
            .value(component, variable)
            .unwrap_or_default();
        let sampled: Vec<SampledValue> = list
            .split(',')
            .filter_map(|name| serde_json::from_value(Value::String(name.to_owned())).ok())
            .flat_map(|measurand| self.sampled(now, index, measurand, context))
            .collect();
        (!sampled.is_empty()).then_some(MeterValue {
            sampled_value: sampled,
            timestamp: DateTime { unix_ms: now },
        })
    }

    pub(super) fn interval_ms(&self, component: &str, variable: &str) -> Option<u64> {
        let seconds = self.device_model.number(component, variable)?;
        Some(seconds * 1000).filter(|&ms| ms > 0)
    }

    fn sampled(
        &self,
        now: u64,
        index: usize,
        measurand: Measurand,
        context: ReadingContext,
    ) -> Vec<SampledValue> {
        let connector = &self.station.connectors[index];
        let sample = connector.sample(now);
        let lines = electrical::lines(&self.station, index, &sample, now);
        let value = |value: f64, phase: Option<Phase>, side: Side, unit: &str| SampledValue {
            value,
            context: Some(context),
            measurand: Some(measurand),
            phase,
            location: Some(location(side)),
            signed_meter_value: None,
            unit_of_measure: Some(UnitOfMeasure {
                unit: CiString::new(unit).ok(),
                multiplier: None,
            }),
        };
        let total = |number: f64, unit: &str| value(number, None, Side::Outlet, unit);
        let round =
            |number: f64, places: i32| (number * 10f64.powi(places)).round() / 10f64.powi(places);
        let offered = connector.offered_w();
        match measurand {
            Measurand::EnergyActiveImportRegister => vec![total(sample.register_wh.round(), "Wh")],
            Measurand::PowerActiveImport => [Side::Outlet, Side::Inlet]
                .into_iter()
                .filter_map(|side| {
                    let watts = electrical::power_w(&lines, side)?;
                    Some(value(watts.round(), None, side, "W"))
                })
                .collect(),
            Measurand::PowerOffered => offered
                .map(|watts| total(watts.round(), "W"))
                .into_iter()
                .collect(),
            Measurand::CurrentOffered => offered
                .map(|watts| electrical::offered_a(&self.station, index, &sample, watts))
                .map(|amps| total(round(amps, 1), "A"))
                .into_iter()
                .collect(),
            Measurand::CurrentImport => lines
                .iter()
                .map(|line| {
                    let phase = line.phase.map(|phase| LINES[phase].0);
                    value(round(line.current_a, 1), phase, line.side, "A")
                })
                .collect(),
            Measurand::Voltage => lines
                .iter()
                .map(|line| {
                    let phase = line.phase.map(|phase| LINES[phase].1);
                    value(round(line.voltage_v, 1), phase, line.side, "V")
                })
                .collect(),
            Measurand::SoC if connector.plugged.is_some() => {
                vec![total(round(sample.soc * 100.0, 1), "Percent")]
            }
            _ => Vec::new(),
        }
    }
}

fn location(side: Side) -> Location {
    match side {
        Side::Outlet => Location::Outlet,
        Side::Inlet => Location::Inlet,
    }
}

#[cfg(test)]
#[path = "metering.test.rs"]
mod tests;
