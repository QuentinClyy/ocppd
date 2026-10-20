use super::ChargePoint;
use super::transaction::Transaction;
use crate::v16::messages::datatypes::meter_value::ReadingContext;
use crate::v16::messages::firmware::{
    DiagnosticsStatusNotificationRequest, FirmwareStatusNotificationRequest,
};
use crate::v16::messages::remote_trigger::{
    ExtendedMessageTrigger, ExtendedTriggerMessageRequest, MessageTrigger, TriggerMessageRequest,
    TriggerMessageStatus,
};
use serde_json::json;

impl ChargePoint {
    pub(super) fn trigger(
        &mut self,
        now: u64,
        request: TriggerMessageRequest,
    ) -> TriggerMessageStatus {
        let connectors = self.station.connectors.len() as u32;
        let target = request.connector_id;
        if target.is_some_and(|connector| connector > connectors) {
            return TriggerMessageStatus::Rejected;
        }
        match request.requested_message {
            MessageTrigger::BootNotification => {
                self.outbox
                    .push("BootNotification", 0, json!(self.boot_request))
            }
            MessageTrigger::Heartbeat => self.heartbeat(now),
            MessageTrigger::StatusNotification => {
                let all = 0..=connectors;
                for connector in target.map_or(all, |connector| connector..=connector) {
                    self.trigger_status(connector);
                }
            }
            MessageTrigger::MeterValues => match target {
                Some(0) => return TriggerMessageStatus::Rejected,
                Some(connector) => self.trigger_meter_values(now, connector as usize - 1),
                None => {
                    for index in 0..connectors as usize {
                        self.trigger_meter_values(now, index);
                    }
                }
            },
            MessageTrigger::DiagnosticsStatusNotification => {
                let request = DiagnosticsStatusNotificationRequest {
                    status: self.current_diagnostics_status(),
                };
                self.outbox
                    .push("DiagnosticsStatusNotification", 0, json!(request));
            }
            MessageTrigger::FirmwareStatusNotification => {
                let request = FirmwareStatusNotificationRequest {
                    status: self.current_firmware_status(),
                };
                self.outbox
                    .push("FirmwareStatusNotification", 0, json!(request));
            }
        }
        TriggerMessageStatus::Accepted
    }

    pub(super) fn extended_trigger(
        &mut self,
        now: u64,
        request: ExtendedTriggerMessageRequest,
    ) -> TriggerMessageStatus {
        let requested_message = match request.requested_message {
            ExtendedMessageTrigger::BootNotification => MessageTrigger::BootNotification,
            ExtendedMessageTrigger::Heartbeat => MessageTrigger::Heartbeat,
            ExtendedMessageTrigger::MeterValues => MessageTrigger::MeterValues,
            ExtendedMessageTrigger::StatusNotification => MessageTrigger::StatusNotification,
            ExtendedMessageTrigger::FirmwareStatusNotification
            | ExtendedMessageTrigger::LogStatusNotification
            | ExtendedMessageTrigger::SignChargePointCertificate => {
                return TriggerMessageStatus::NotImplemented;
            }
        };
        let request = TriggerMessageRequest {
            requested_message,
            connector_id: request.connector_id,
        };
        self.trigger(now, request)
    }

    fn trigger_status(&mut self, connector: u32) {
        if let Some(status) = self.reported[connector as usize] {
            self.status_notification(connector, status);
        }
    }

    fn trigger_meter_values(&mut self, now: u64, index: usize) {
        let (transaction_id, last_register_wh) = match self.transactions[index] {
            Transaction::Active {
                transaction_id,
                last_register_wh,
                ..
            } => (transaction_id, last_register_wh),
            _ => (None, self.station.connectors[index].sample(now).register_wh),
        };
        let reading = self.reading(index, now, last_register_wh, ReadingContext::Trigger);
        let measurands = self.configuration.measurands();
        self.push_meter_values(index, now, transaction_id, &reading, &measurands);
    }
}

#[cfg(test)]
#[path = "trigger.test.rs"]
mod tests;
