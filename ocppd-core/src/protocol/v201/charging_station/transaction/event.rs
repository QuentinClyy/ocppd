use super::Authorization;
use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::component::Evse;
use crate::protocol::v201::messages::datatypes::meter_value::MeterValue;
use crate::protocol::v201::messages::transactions::{
    TransactionEventRequest, TransactionEventType, TransactionInfo, TriggerReason,
};
use crate::wire::date_time::DateTime;
use serde_json::json;

const CONNECTOR_ID: u32 = 1;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn send_transaction_event(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        event_type: TransactionEventType,
        trigger_reason: TriggerReason,
        meter_value: Vec<MeterValue>,
    ) {
        let charging_state = self.charging_state(now, index);
        let online = self.is_accepted();
        let session = &mut self.sessions[index];
        let id_token = match &session.authorization {
            Authorization::Authorized { id_token, .. } => Some(id_token.clone()),
            _ => None,
        };
        let Some(transaction) = &mut session.transaction else {
            return;
        };
        let remote_start_id = session.remote_start_id.take();
        let reservation_id = session.reservation_id.take();
        let send_id_token = id_token.filter(|_| !transaction.id_token_sent);
        transaction.id_token_sent |= send_id_token.is_some();
        let state_changed = charging_state != transaction.charging_state;
        transaction.charging_state = charging_state;
        let ended = event_type == TransactionEventType::Ended;
        let evse = index as u32 + 1;
        let request = TransactionEventRequest {
            event_type,
            meter_value: (!meter_value.is_empty()).then_some(meter_value),
            timestamp: DateTime { unix_ms: now },
            trigger_reason,
            seq_no: transaction.seq_no,
            offline: (!online).then_some(true),
            number_of_phases_used: None,
            cable_max_current: None,
            reservation_id,
            transaction_info: TransactionInfo {
                transaction_id: transaction.id.clone(),
                charging_state: (event_type == TransactionEventType::Started
                    || state_changed
                    || trigger_reason == TriggerReason::Trigger)
                    .then_some(charging_state),
                time_spent_charging: None,
                stopped_reason: transaction.stop_reason.filter(|_| ended),
                remote_start_id,
            },
            evse: Some(Evse {
                id: evse,
                connector_id: Some(CONNECTOR_ID),
            }),
            id_token: send_id_token,
        };
        transaction.seq_no += 1;
        let transaction_id = transaction.id.as_str().to_owned();
        self.outbox.push("TransactionEvent", evse, json!(request));
        let kind = EventKind::TransactionEvent {
            evse,
            transaction_id,
            event_type,
            trigger_reason,
        };
        self.emit(out, now, kind);
    }
}
