use super::charge_point::ChargePoint;
use super::messages::meter_value::MeterValuesRequest;
use super::messages::status::ChargePointStatus;
use super::meter_values::meter_value;
use super::transaction::{METER_VALUE_SAMPLE_INTERVAL_MS, Transaction};
use crate::event::{EventKind, Output};
use serde_json::json;

impl ChargePoint {
    pub(super) fn sample_meters(&mut self, out: &mut Vec<Output>, now: u64) {
        for index in 0..self.transactions.len() {
            let Transaction::Active {
                transaction_id,
                id_tag,
                next_sample,
            } = self.transactions[index].clone()
            else {
                continue;
            };
            if now < next_sample {
                continue;
            }
            let sample = self.station.connectors[index].sample(now);
            let request = MeterValuesRequest {
                connector_id: index as u32 + 1,
                transaction_id: Some(transaction_id),
                meter_value: vec![meter_value(now, &sample, &self.station)],
            };
            self.outbox
                .push("MeterValues", index as u32 + 1, json!(request));
            let kind = EventKind::MeterSample {
                connector: index as u32 + 1,
                register_wh: sample.register_wh,
                power_w: sample.power_w,
                soc: sample.soc,
            };
            self.emit(out, now, kind);
            self.transactions[index] = Transaction::Active {
                transaction_id,
                id_tag,
                next_sample: now + METER_VALUE_SAMPLE_INTERVAL_MS,
            };
        }
    }

    pub(super) fn transaction_deadline(&self) -> Option<u64> {
        let mut deadlines = Vec::new();
        for (index, transaction) in self.transactions.iter().enumerate() {
            if let Transaction::Active { next_sample, .. } = transaction {
                deadlines.push(*next_sample);
            }
            if self.reported[index] == Some(ChargePointStatus::Charging) {
                deadlines.extend(self.station.connectors[index].finish_time());
            }
        }
        deadlines.into_iter().min()
    }

    pub(super) fn report_statuses(&mut self, out: &mut Vec<Output>, now: u64) {
        for index in 0..self.transactions.len() {
            let status = self.status(index, now);
            if self.reported[index] == Some(status) {
                continue;
            }
            self.reported[index] = Some(status);
            let connector = index as u32 + 1;
            self.status_notification(connector, status);
            self.emit(out, now, EventKind::StatusChanged { connector, status });
        }
    }

    fn status(&self, index: usize, now: u64) -> ChargePointStatus {
        let connector = &self.station.connectors[index];
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
            Transaction::Active { .. } => ChargePointStatus::SuspendedEv,
            Transaction::Stopping | Transaction::Finished => ChargePointStatus::Finishing,
        }
    }
}
