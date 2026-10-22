use super::{
    ACTIVE_MONITORING_LEVEL, HIGHEST_SEVERITY, MONITORING_CTRLR, Monitor, OFFLINE_QUEUING_SEVERITY,
    next_period,
};
use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::diagnostics::monitoring::{
    EventData, EventNotificationType, EventTrigger, MonitorType, NotifyEventRequest,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::json;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn evaluate_monitors(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
    ) {
        let due = self.monitoring_deadline().is_some_and(|at| at <= now);
        match due {
            true => self.refresh_device_model(now),
            false => self.refresh_state(now),
        }
        let in_transaction = self
            .sessions
            .iter()
            .any(|session| session.transaction.is_some());
        let transaction_changed = in_transaction != self.monitoring.in_transaction;
        self.monitoring.in_transaction = in_transaction;
        if !self.device_model.take_changed() && !transaction_changed && !due {
            return;
        }
        let level = self
            .device_model
            .number(MONITORING_CTRLR, ACTIVE_MONITORING_LEVEL)
            .unwrap_or(u64::from(HIGHEST_SEVERITY));
        let offline_limit = match self.is_accepted() {
            true => None,
            false => {
                let limit = self
                    .device_model
                    .number(MONITORING_CTRLR, OFFLINE_QUEUING_SEVERITY);
                Some(limit.unwrap_or(0))
            }
        };
        let mut events = Vec::new();
        let mut event_id = self.monitoring.next_event_id;
        for monitor in &mut self.monitoring.monitors {
            if monitor.transaction && !in_transaction {
                if monitor.next_at.is_some_and(|at| at <= now) {
                    monitor.next_at = next_period(monitor.kind, monitor.value, now);
                }
                continue;
            }
            let Ok(entry) = self
                .device_model
                .entry(&monitor.component, &monitor.variable)
            else {
                continue;
            };
            let numeric = matches!(
                entry.characteristics.data_type,
                DataType::Integer | DataType::Decimal
            );
            let Some((trigger, cleared)) = check(monitor, &entry.value, numeric, now) else {
                continue;
            };
            let severity = u64::from(monitor.severity);
            let queued = offline_limit.is_none_or(|limit| severity <= limit);
            if !queued || severity > level {
                continue;
            }
            let readable = entry.mutability != Mutability::WriteOnly;
            let actual = if readable { entry.value.as_str() } else { "" };
            let Ok(actual_value) = CiString::new(actual) else {
                continue;
            };
            let session = monitor
                .component
                .evse
                .as_ref()
                .and_then(|evse| self.sessions.get(evse.id as usize - 1));
            let transaction = session.and_then(|session| session.transaction.as_ref());
            event_id += 1;
            events.push(EventData {
                event_id,
                timestamp: DateTime { unix_ms: now },
                trigger,
                actual_value,
                cleared,
                transaction_id: transaction.map(|transaction| transaction.id.clone()),
                component: monitor.component.clone(),
                variable_monitoring_id: monitor.id,
                event_notification_type: EventNotificationType::CustomMonitor,
                variable: monitor.variable.clone(),
            });
        }
        self.monitoring.next_event_id = event_id;
        if events.is_empty() {
            return;
        }
        for event in &events {
            let kind = EventKind::MonitorTriggered {
                monitor_id: event.variable_monitoring_id,
                actual_value: event.actual_value.as_str().to_owned(),
                cleared: event.cleared == Some(true),
            };
            self.emit(out, now, kind);
        }
        let request = NotifyEventRequest {
            generated_at: DateTime { unix_ms: now },
            tbc: false,
            seq_no: 0,
            event_data: events,
        };
        self.outbox.push("NotifyEvent", 0, json!(request));
    }
}

fn check(
    monitor: &mut Monitor,
    value: &str,
    numeric: bool,
    now: u64,
) -> Option<(EventTrigger, Option<bool>)> {
    let number = value.parse::<f64>().ok().filter(|_| numeric);
    match monitor.kind {
        MonitorType::UpperThreshold | MonitorType::LowerThreshold => {
            let number = number?;
            let exceeded = match monitor.kind {
                MonitorType::UpperThreshold => number > monitor.value,
                _ => number < monitor.value,
            };
            if exceeded == monitor.tripped {
                return None;
            }
            monitor.tripped = exceeded;
            Some((EventTrigger::Alerting, (!exceeded).then_some(true)))
        }
        MonitorType::Delta => {
            let before = monitor.reference.as_deref();
            let changed = match (number, before.and_then(|before| before.parse::<f64>().ok())) {
                (Some(number), Some(before)) => (number - before).abs() > monitor.value,
                _ => before != Some(value),
            };
            if !changed {
                return None;
            }
            monitor.reference = Some(value.to_owned());
            Some((EventTrigger::Delta, None))
        }
        MonitorType::Periodic | MonitorType::PeriodicClockAligned => {
            if monitor.next_at.is_none_or(|at| now < at) {
                return None;
            }
            monitor.next_at = next_period(monitor.kind, monitor.value, now);
            Some((EventTrigger::Periodic, None))
        }
    }
}

#[cfg(test)]
#[path = "alert.test.rs"]
mod tests;
