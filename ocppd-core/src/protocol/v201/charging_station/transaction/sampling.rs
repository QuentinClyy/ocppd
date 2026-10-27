use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::metering::{
    ALIGNED_DATA_CTRLR, SAMPLED_DATA_CTRLR, TX_ENDED_MEASURANDS,
};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::meter_value::{MeterValue, ReadingContext};
use crate::protocol::v201::messages::transaction::{TransactionEventType, TriggerReason};

const TX_STARTED_MEASURANDS: &str = "TxStartedMeasurands";
const TX_UPDATED_MEASURANDS: &str = "TxUpdatedMeasurands";
const TX_UPDATED_INTERVAL: &str = "TxUpdatedInterval";
const TX_ENDED_INTERVAL: &str = "TxEndedInterval";

impl ChargingStation {
    pub(super) fn started_sample(&self, now: u64, index: usize) -> Vec<MeterValue> {
        let measurands = (SAMPLED_DATA_CTRLR, TX_STARTED_MEASURANDS);
        let context = ReadingContext::TransactionBegin;
        self.meter_value(now, index, measurands, context)
            .into_iter()
            .collect()
    }

    pub(super) fn ended_sample(&self, now: u64, index: usize, last: bool) -> Vec<MeterValue> {
        let measurands = (SAMPLED_DATA_CTRLR, TX_ENDED_MEASURANDS);
        let context = match last {
            true => ReadingContext::TransactionEnd,
            false => ReadingContext::TransactionBegin,
        };
        self.meter_value(now, index, measurands, context)
            .into_iter()
            .collect()
    }

    pub(super) fn ended_samples(&mut self, now: u64, index: usize) -> Vec<MeterValue> {
        let end = self.ended_sample(now, index, true);
        let Some(transaction) = &mut self.sessions[index].transaction else {
            return end;
        };
        let mut samples = std::mem::take(&mut transaction.ended_samples);
        samples.extend(end);
        samples
    }

    pub(super) fn sample_transaction(&mut self, out: &mut Vec<Output>, now: u64, index: usize) {
        let updated = self.interval_ms(SAMPLED_DATA_CTRLR, TX_UPDATED_INTERVAL);
        let ended = self.interval_ms(SAMPLED_DATA_CTRLR, TX_ENDED_INTERVAL);
        let clock = self.interval_ms(ALIGNED_DATA_CTRLR, TX_ENDED_INTERVAL);
        let Some(transaction) = &self.sessions[index].transaction else {
            return;
        };
        let clock_due =
            clock.is_some_and(|interval| now / interval > transaction.last_clock_sample / interval);
        let periodic_due =
            updated.is_some_and(|interval| now >= transaction.last_periodic + interval);
        let sample_due =
            ended.is_some_and(|interval| now >= transaction.last_ended_sample + interval);
        if periodic_due {
            let measurands = (SAMPLED_DATA_CTRLR, TX_UPDATED_MEASURANDS);
            let periodic = self.meter_value(now, index, measurands, ReadingContext::SamplePeriodic);
            if let Some(transaction) = &mut self.sessions[index].transaction {
                transaction.last_periodic = now;
            }
            let updated = TransactionEventType::Updated;
            let trigger = TriggerReason::MeterValuePeriodic;
            self.send_transaction_event(
                out,
                now,
                index,
                updated,
                trigger,
                periodic.into_iter().collect(),
            );
            let sample = self.station.connectors[index].sample(now);
            let kind = EventKind::MeterSample {
                evse: index as u32 + 1,
                register_wh: sample.register_wh,
                power_w: sample.power_w,
                soc: sample.soc,
            };
            self.emit(out, now, kind);
        }
        if sample_due {
            let measurands = (SAMPLED_DATA_CTRLR, TX_ENDED_MEASURANDS);
            let sample = self.meter_value(now, index, measurands, ReadingContext::SamplePeriodic);
            if let Some(transaction) = &mut self.sessions[index].transaction {
                transaction.last_ended_sample = now;
                transaction.ended_samples.extend(sample);
            }
        }
        if clock_due {
            let measurands = (ALIGNED_DATA_CTRLR, TX_ENDED_MEASURANDS);
            let sample = self.meter_value(now, index, measurands, ReadingContext::SampleClock);
            if let Some(transaction) = &mut self.sessions[index].transaction {
                transaction.last_clock_sample = now;
                transaction.ended_samples.extend(sample);
            }
        }
    }

    pub(in crate::protocol::v201::charging_station) fn sampling_deadline(&self) -> Option<u64> {
        let updated = self.interval_ms(SAMPLED_DATA_CTRLR, TX_UPDATED_INTERVAL);
        let ended = self.interval_ms(SAMPLED_DATA_CTRLR, TX_ENDED_INTERVAL);
        let clock = self.interval_ms(ALIGNED_DATA_CTRLR, TX_ENDED_INTERVAL);
        self.sessions
            .iter()
            .filter_map(|session| session.transaction.as_ref())
            .flat_map(|transaction| {
                let periodic = updated.map(|interval| transaction.last_periodic + interval);
                let sample = ended.map(|interval| transaction.last_ended_sample + interval);
                let boundary =
                    clock.map(|interval| (transaction.last_clock_sample / interval + 1) * interval);
                periodic.into_iter().chain(sample).chain(boundary)
            })
            .min()
    }
}
