use super::charge_point::ChargePoint;
use super::configuration::METER_VALUE_SAMPLE_INTERVAL;
use super::messages::meter_value::MeterValuesRequest;
use super::messages::status::ChargePointStatus;
use super::meter_values::meter_value;
use super::transaction::Transaction;
use crate::event::{EventKind, Output};
use serde_json::json;

impl ChargePoint {
    pub(super) fn sample_meters(&mut self, out: &mut Vec<Output>, now: u64) {
        let Some(interval) = self.configuration.interval_ms(METER_VALUE_SAMPLE_INTERVAL) else {
            return;
        };
        let measurands = self.configuration.measurands();
        for index in 0..self.transactions.len() {
            let Transaction::Active {
                transaction_id,
                id_tag,
                last_sample,
            } = self.transactions[index].clone()
            else {
                continue;
            };
            if now < last_sample + interval {
                continue;
            }
            let sample = self.station.connectors[index].sample(now);
            if !measurands.is_empty() {
                let request = MeterValuesRequest {
                    connector_id: index as u32 + 1,
                    transaction_id: Some(transaction_id),
                    meter_value: vec![meter_value(now, &sample, &self.station, &measurands)],
                };
                self.outbox
                    .push("MeterValues", index as u32 + 1, json!(request));
            }
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
                last_sample: now,
            };
        }
    }

    pub(super) fn transaction_deadline(&self) -> Option<u64> {
        let sample_interval = self.configuration.interval_ms(METER_VALUE_SAMPLE_INTERVAL);
        let mut deadlines = Vec::new();
        for (index, transaction) in self.transactions.iter().enumerate() {
            if let (Transaction::Active { last_sample, .. }, Some(interval)) =
                (transaction, sample_interval)
            {
                deadlines.push(last_sample + interval);
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
