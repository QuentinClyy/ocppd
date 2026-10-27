use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::metering::{
    ALIGNED_DATA_CTRLR, MEASURANDS_VARIABLE, SAMPLED_DATA_CTRLR,
};
use crate::protocol::v201::messages::availability::{HeartbeatRequest, StatusNotificationRequest};
use crate::protocol::v201::messages::datatypes::meter_value::ReadingContext;
use crate::protocol::v201::messages::meter_values::MeterValuesRequest;
use crate::protocol::v201::messages::provisioning::boot::BootReason;
use crate::protocol::v201::messages::remote_control::{
    MessageTrigger, TriggerMessageRequest, TriggerMessageStatus,
};
use crate::protocol::v201::messages::transaction::{TransactionEventType, TriggerReason};
use crate::wire::date_time::DateTime;
use serde_json::json;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn trigger_message(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: TriggerMessageRequest,
    ) -> TriggerMessageStatus {
        let count = self.sessions.len();
        let evse = request.evse.map(|evse| evse.id);
        if evse.is_some_and(|id| id == 0 || id as usize > count) {
            return TriggerMessageStatus::Rejected;
        }
        let indexes: Vec<usize> = match evse {
            Some(id) => vec![id as usize - 1],
            None => (0..count).collect(),
        };
        match request.requested_message {
            MessageTrigger::BootNotification if self.is_accepted() => {
                TriggerMessageStatus::Rejected
            }
            MessageTrigger::BootNotification => {
                self.boot_request.reason = BootReason::Triggered;
                self.outbox
                    .push("BootNotification", 0, json!(self.boot_request));
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::Heartbeat => {
                self.outbox.push("Heartbeat", 0, json!(HeartbeatRequest {}));
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::StatusNotification => {
                let connector = request.evse.and_then(|evse| evse.connector_id);
                let (Some(index), Some(1)) = (evse.map(|id| id as usize - 1), connector) else {
                    return TriggerMessageStatus::Rejected;
                };
                let request = StatusNotificationRequest {
                    timestamp: DateTime { unix_ms: now },
                    connector_status: self.connector_status(index),
                    evse_id: index as u32 + 1,
                    connector_id: 1,
                };
                self.outbox
                    .push("StatusNotification", index as u32 + 1, json!(request));
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::MeterValues => {
                let measurands = (ALIGNED_DATA_CTRLR, MEASURANDS_VARIABLE);
                let samples: Vec<_> = indexes
                    .iter()
                    .filter_map(|&index| {
                        let sample =
                            self.meter_value(now, index, measurands, ReadingContext::Trigger)?;
                        Some((index, sample))
                    })
                    .collect();
                if samples.is_empty() {
                    return TriggerMessageStatus::Rejected;
                }
                for (index, sample) in samples {
                    let evse_id = index as u32 + 1;
                    let request = MeterValuesRequest {
                        evse_id,
                        meter_value: vec![sample],
                    };
                    self.outbox.push("MeterValues", evse_id, json!(request));
                }
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::TransactionEvent => {
                let running: Vec<usize> = indexes
                    .into_iter()
                    .filter(|&index| self.sessions[index].transaction.is_some())
                    .collect();
                if running.is_empty() {
                    return TriggerMessageStatus::Rejected;
                }
                let measurands = (SAMPLED_DATA_CTRLR, "TxUpdatedMeasurands");
                for index in running {
                    let sample = self.meter_value(now, index, measurands, ReadingContext::Trigger);
                    let updated = TransactionEventType::Updated;
                    let trigger = TriggerReason::Trigger;
                    self.send_transaction_event(
                        out,
                        now,
                        index,
                        updated,
                        trigger,
                        sample.into_iter().collect(),
                    );
                }
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::LogStatusNotification => {
                self.report_log_status();
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::FirmwareStatusNotification => {
                self.report_firmware_status();
                TriggerMessageStatus::Accepted
            }
            MessageTrigger::SignChargingStationCertificate => {
                match self.sign_charging_station_certificate() {
                    true => TriggerMessageStatus::Accepted,
                    false => TriggerMessageStatus::Rejected,
                }
            }
            _ => TriggerMessageStatus::NotImplemented,
        }
    }
}

#[cfg(test)]
#[path = "trigger.test.rs"]
mod tests;
