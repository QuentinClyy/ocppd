use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::messages::datatypes::component::{Component, Evse, Variable};
use crate::protocol::v201::messages::diagnostics::monitoring::{
    EventData, EventNotificationType, EventTrigger, NotifyEventRequest,
};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::json;

pub const LOCK: &str = "ConnectorPlugRetentionLock";
pub const PROBLEM: &str = "Problem";

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn lock_failed(
        &mut self,
        now: u64,
        index: usize,
    ) {
        let evse = lock_of(index);
        let value = "true".to_owned();
        self.device_model.set_value(LOCK, evse, PROBLEM, value);
        self.monitoring.next_event_id += 1;
        let event = EventData {
            event_id: self.monitoring.next_event_id,
            timestamp: DateTime { unix_ms: now },
            trigger: EventTrigger::Delta,
            actual_value: name("true"),
            cleared: None,
            transaction_id: None,
            component: Component {
                evse,
                name: name(LOCK),
                instance: None,
            },
            variable_monitoring_id: None,
            event_notification_type: EventNotificationType::HardWiredNotification,
            variable: Variable {
                name: name(PROBLEM),
                instance: None,
            },
        };
        let request = NotifyEventRequest {
            generated_at: DateTime { unix_ms: now },
            tbc: false,
            seq_no: 0,
            event_data: vec![event],
        };
        self.outbox.push("NotifyEvent", 0, json!(request));
    }

    pub(in crate::protocol::v201::charging_station) fn lock_repaired(&mut self, index: usize) {
        let value = "false".to_owned();
        self.device_model
            .set_value(LOCK, lock_of(index), PROBLEM, value);
    }
}

fn lock_of(index: usize) -> Option<Evse> {
    Some(Evse {
        id: index as u32 + 1,
        connector_id: Some(1),
    })
}

fn name<const N: usize>(text: &str) -> CiString<N> {
    CiString::new(text).unwrap_or_else(|_| unreachable!("a constant shorter than its limit"))
}
