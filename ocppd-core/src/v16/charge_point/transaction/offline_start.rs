use super::Transaction;
use crate::api::event::EventKind;
use crate::api::io::Output;
use crate::v16::charge_point::ChargePoint;
use crate::v16::messages::core::transaction::{Reason, StartTransactionResponse};
use crate::v16::messages::datatypes::id_tag::AuthorizationStatus;
use serde_json::json;

impl ChargePoint {
    pub(crate) fn started_offline(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        response: StartTransactionResponse,
    ) {
        let Transaction::Active {
            transaction_id: None,
            id_tag,
            ..
        } = self.transactions[index].clone()
        else {
            return;
        };
        self.remember(&id_tag, &response.id_tag_info);
        let transaction_id = response.transaction_id;
        self.fill_transaction_id(index, transaction_id);
        if let Transaction::Active {
            transaction_id: id, ..
        } = &mut self.transactions[index]
        {
            *id = Some(transaction_id);
        }
        let kind = EventKind::TransactionStarted {
            connector: index as u32 + 1,
            transaction_id,
        };
        self.emit(out, now, kind);
        if response.id_tag_info.status != AuthorizationStatus::Accepted {
            self.stop(out, now, index, Reason::DeAuthorized);
        }
    }

    pub(crate) fn fill_transaction_id(&mut self, index: usize, transaction_id: i32) -> bool {
        let mut stop_filled = false;
        for (action, payload) in self.outbox.queued_payloads(index as u32 + 1) {
            if action == "StartTransaction" {
                break;
            }
            let Some(fields) = payload.as_object_mut() else {
                continue;
            };
            let waiting = matches!(action, "MeterValues" | "StopTransaction")
                && !fields.contains_key("transactionId");
            if waiting {
                fields.insert("transactionId".into(), json!(transaction_id));
                stop_filled |= action == "StopTransaction";
            }
        }
        stop_filled
    }
}

#[cfg(test)]
#[path = "offline_start.test.rs"]
mod tests;
