use super::Authorization;
use crate::api::event::LinkEvent;
use crate::api::io::Output;
use crate::v201::charging_station::ChargingStation;
use crate::v201::charging_station::device_model::table::{OCPP_COMM_CTRLR, TX_CTRLR};
use crate::v201::event::EventKind;
use crate::v201::messages::datatypes::id_token::AuthorizationStatus;
use crate::v201::messages::transactions::{
    GetTransactionStatusRequest, GetTransactionStatusResponse, TransactionEventResponse,
    TriggerReason,
};
use crate::wire::outbox::{Call, Sent};
use serde_json::{Value, json};

const TRANSACTION_EVENT: &str = "TransactionEvent";

impl ChargingStation {
    pub(in crate::v201::charging_station) fn transaction_reply(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        call: &Call,
        payload: Value,
    ) {
        let response =
            self.decode::<TransactionEventResponse>(out, now, TRANSACTION_EVENT, payload);
        let status = response
            .and_then(|response| response.id_token_info)
            .map(|info| info.status);
        let index = call.connector_id as usize - 1;
        let refused = status.is_some_and(|status| status != AuthorizationStatus::Accepted);
        let running = self.sessions[index].transaction.is_some();
        if !refused || !running {
            return;
        }
        match self.device_model.value(TX_CTRLR, "StopTxOnInvalidId") == Some("true") {
            true => {
                self.sessions[index].authorization = Authorization::None;
                self.settle(out, now, index, TriggerReason::Deauthorized);
            }
            false => {
                self.sessions[index].suspended = true;
                self.settle(out, now, index, TriggerReason::ChargingStateChanged);
            }
        }
    }

    pub(in crate::v201::charging_station) fn retry_transaction_event(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        sent: Sent,
    ) {
        let action = sent.call.action.clone();
        let attempts = self.message_setting("MessageAttempts").unwrap_or(1);
        if u64::from(sent.attempts) >= attempts {
            return self.emit(
                out,
                now,
                EventKind::Link(LinkEvent::CallAbandoned { action }),
            );
        }
        let interval = self.message_setting("MessageAttemptInterval").unwrap_or(0) * 1000;
        let attempt = sent.attempts;
        let retry_at = now + interval * u64::from(attempt);
        let kind = LinkEvent::CallRetried {
            action,
            attempt,
            retry_at,
        };
        self.emit(out, now, EventKind::Link(kind));
        self.outbox.retry(sent, retry_at);
    }

    pub(in crate::v201::charging_station) fn mark_queued_offline(&mut self) {
        for evse in 1..=self.station.connectors.len() as u32 {
            for (action, payload) in self.outbox.queued_payloads(evse) {
                if action == TRANSACTION_EVENT {
                    payload["offline"] = json!(true);
                }
            }
        }
    }

    pub(in crate::v201::charging_station) fn transaction_status(
        &self,
        request: GetTransactionStatusRequest,
    ) -> GetTransactionStatusResponse {
        let Some(id) = request.transaction_id else {
            let queued = self.outbox.holds(|action, _| action == TRANSACTION_EVENT);
            return GetTransactionStatusResponse {
                ongoing_indicator: None,
                messages_in_queue: queued,
            };
        };
        let ongoing = self.sessions.iter().any(|session| {
            session
                .transaction
                .as_ref()
                .is_some_and(|transaction| transaction.id.matches(&id))
        });
        let queued = self.outbox.holds(|action, payload| {
            action == TRANSACTION_EVENT
                && payload["transactionInfo"]["transactionId"]
                    .as_str()
                    .is_some_and(|queued| queued.eq_ignore_ascii_case(id.as_str()))
        });
        GetTransactionStatusResponse {
            ongoing_indicator: Some(ongoing),
            messages_in_queue: queued,
        }
    }

    fn message_setting(&self, variable: &str) -> Option<u64> {
        let value =
            self.device_model
                .instance_value(OCPP_COMM_CTRLR, variable, Some(TRANSACTION_EVENT))?;
        value.parse().ok()
    }
}

#[cfg(test)]
#[path = "reliability.test.rs"]
mod tests;
