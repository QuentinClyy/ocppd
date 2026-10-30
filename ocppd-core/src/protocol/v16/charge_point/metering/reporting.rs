use super::meter_values::{Reading, meter_value};
use crate::api::io::Output;
use crate::protocol::v16::charge_point::ChargePoint;
use crate::protocol::v16::charge_point::provisioning::configuration::METER_VALUE_SAMPLE_INTERVAL;
use crate::protocol::v16::charge_point::transaction::Transaction;
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use crate::protocol::v16::messages::datatypes::meter_value::{Measurand, ReadingContext};
use crate::protocol::v16::messages::meter_values::MeterValuesRequest;
use crate::station::electrical;
use serde_json::json;

impl ChargePoint {
    pub(crate) fn sample_meters(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(interval) = self.configuration.interval_ms(METER_VALUE_SAMPLE_INTERVAL) else {
            return;
        };
        let measurands = self.configuration.measurands();
        for index in 0..self.transactions.len() {
            while let Transaction::Active { last_sample, .. } = self.transactions[index] {
                let due = last_sample + interval;
                if now < due {
                    break;
                }
                self.sample_meter(out, index, due, &measurands);
            }
        }
    }

    fn sample_meter(
        &mut self,
        out: &mut Vec<Output>,
        index: usize,
        at: u64,
        measurands: &[Measurand],
    ) {
        let Transaction::Active {
            transaction_id,
            last_register_wh,
            ..
        } = self.transactions[index]
        else {
            return;
        };
        let reading = self.reading(index, at, last_register_wh, ReadingContext::SamplePeriodic);
        let sample = reading.sample;
        self.push_meter_values(index, at, transaction_id, &reading, measurands);
        let kind = EventKind::MeterSample {
            connector: index as u32 + 1,
            register_wh: sample.register_wh,
            power_w: sample.power_w,
            soc: sample.soc,
        };
        self.emit(out, at, kind);
        if let Transaction::Active {
            last_sample,
            last_register_wh,
            ..
        } = &mut self.transactions[index]
        {
            *last_sample = at;
            *last_register_wh = sample.register_wh;
        }
    }

    pub(crate) fn reading(
        &self,
        index: usize,
        at: u64,
        last_register_wh: f64,
        context: ReadingContext,
    ) -> Reading {
        let connector = &self.station.connectors[index];
        let sample = connector.sample(at);
        let offered_w = connector.offered_w().unwrap_or(self.station.limit_w());
        Reading {
            sample,
            lines: electrical::lines(&self.station, index, &sample, at),
            offered_w,
            offered_a: electrical::offered_a(&self.station, index, &sample, offered_w),
            soc: connector.plugged.as_ref().map(|_| sample.soc),
            interval_wh: sample.register_wh - last_register_wh,
            context,
        }
    }

    pub(crate) fn push_meter_values(
        &mut self,
        index: usize,
        at: u64,
        transaction_id: Option<i32>,
        reading: &Reading,
        measurands: &[Measurand],
    ) {
        if measurands.is_empty() {
            return;
        }
        let request = MeterValuesRequest {
            connector_id: index as u32 + 1,
            transaction_id,
            meter_value: vec![meter_value(at, reading, measurands)],
        };
        self.outbox
            .push("MeterValues", index as u32 + 1, json!(request));
    }

    pub(crate) fn transaction_deadline(&self) -> Option<u64> {
        let sample_interval = self.configuration.interval_ms(METER_VALUE_SAMPLE_INTERVAL);
        let mut deadlines = Vec::new();
        for (index, transaction) in self.transactions.iter().enumerate() {
            if let (Transaction::Active { last_sample, .. }, Some(interval)) =
                (transaction, sample_interval)
            {
                deadlines.push(last_sample + interval);
            }
            if self.reported[index + 1] == Some(ChargePointStatus::Charging) {
                deadlines.extend(self.station.connectors[index].finish_time());
            }
        }
        deadlines.into_iter().min()
    }

    pub(crate) fn report_statuses(&mut self, out: &mut Vec<Output>, now: u64) {
        for connector_id in 0..self.reported.len() {
            let status = match connector_id {
                0 if self.inoperative[0] => ChargePointStatus::Unavailable,
                0 => ChargePointStatus::Available,
                _ => self.status(connector_id - 1, now),
            };
            let changed = self.reported[connector_id] != Some(status);
            if !changed && !self.resend_statuses {
                continue;
            }
            self.reported[connector_id] = Some(status);
            let connector = connector_id as u32;
            self.status_notification(connector, status);
            if changed {
                self.emit(out, now, EventKind::StatusChanged { connector, status });
            }
        }
        self.resend_statuses = false;
    }

    pub(crate) fn unavailable(&self, index: usize) -> bool {
        self.inoperative[0] || self.inoperative[index + 1] || self.faulted(index)
    }

    pub(crate) fn status(&self, index: usize, now: u64) -> ChargePointStatus {
        let connector = &self.station.connectors[index];
        if self.faulted(index) {
            return ChargePointStatus::Faulted;
        }
        let in_transaction = matches!(
            self.transactions[index],
            Transaction::Authorizing { .. }
                | Transaction::Starting { .. }
                | Transaction::Active { .. }
        );
        if self.unavailable(index) && !in_transaction {
            return ChargePointStatus::Unavailable;
        }
        if connector.plugged.is_none() && self.held_for(index) {
            return ChargePointStatus::Preparing;
        }
        let deciding = matches!(
            self.transactions[index],
            Transaction::Idle | Transaction::Authorizing { .. }
        );
        if self.is_reserved(index) && deciding {
            return ChargePointStatus::Reserved;
        }
        if connector.plugged.is_none() {
            return ChargePointStatus::Available;
        }
        match self.transactions[index] {
            Transaction::Idle | Transaction::Authorizing { .. } | Transaction::Starting { .. } => {
                ChargePointStatus::Preparing
            }
            Transaction::Active { .. } if connector.sample(now).power_w > 0.0 => {
                ChargePointStatus::Charging
            }
            Transaction::Active { .. } if connector.offered_w() == Some(0.0) => {
                ChargePointStatus::SuspendedEvse
            }
            Transaction::Active { .. } => ChargePointStatus::SuspendedEv,
            Transaction::Finished => ChargePointStatus::Finishing,
        }
    }
}
