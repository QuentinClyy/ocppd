mod alert;

use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::Missing;
use crate::protocol::v201::messages::datatypes::component::{Component, Variable};
use crate::protocol::v201::messages::diagnostics::monitoring::{
    ClearMonitoringResult, ClearMonitoringStatus, ClearVariableMonitoringRequest,
    ClearVariableMonitoringResponse, MonitorType, SetMonitoringData, SetMonitoringResult,
    SetMonitoringStatus, SetVariableMonitoringRequest, SetVariableMonitoringResponse,
};
use crate::protocol::v201::messages::provisioning::report::DataType;

pub const MONITORING_CTRLR: &str = "MonitoringCtrlr";
pub const OFFLINE_QUEUING_SEVERITY: &str = "OfflineMonitoringEventQueuingSeverity";
pub const ITEMS_PER_MONITORING: usize = 50;
const HIGHEST_SEVERITY: u32 = 9;

#[derive(Default)]
pub struct Monitoring {
    monitors: Vec<Monitor>,
    next_id: i32,
    next_event_id: i32,
    in_transaction: bool,
}

struct Monitor {
    id: i32,
    component: Component,
    variable: Variable,
    kind: MonitorType,
    value: f64,
    severity: u32,
    transaction: bool,
    tripped: bool,
    reference: Option<String>,
}

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn set_variable_monitoring(
        &mut self,
        now: u64,
        request: SetVariableMonitoringRequest,
    ) -> SetVariableMonitoringResponse {
        self.refresh_device_model(now);
        let results = request.set_monitoring_data.into_iter();
        let results = results.map(|data| self.set_monitor(data)).collect();
        self.device_model.mark_changed();
        SetVariableMonitoringResponse {
            set_monitoring_result: results,
        }
    }

    pub(in crate::protocol::v201::charging_station) fn clear_variable_monitoring(
        &mut self,
        request: ClearVariableMonitoringRequest,
    ) -> ClearVariableMonitoringResponse {
        let results = request.id.into_iter().map(|id| {
            let monitors = &mut self.monitoring.monitors;
            let before = monitors.len();
            monitors.retain(|monitor| monitor.id != id);
            let status = match monitors.len() < before {
                true => ClearMonitoringStatus::Accepted,
                false => ClearMonitoringStatus::NotFound,
            };
            ClearMonitoringResult {
                status,
                id,
                status_info: None,
            }
        });
        ClearVariableMonitoringResponse {
            clear_monitoring_result: results.collect(),
        }
    }

    fn set_monitor(&mut self, data: SetMonitoringData) -> SetMonitoringResult {
        let status = self.monitor_refusal(&data);
        let id = match status {
            Some(_) => data.id,
            None => Some(data.id.unwrap_or_else(|| self.new_monitor_id())),
        };
        if let (None, Some(id)) = (status, id) {
            let reference = self
                .device_model
                .entry(&data.component, &data.variable)
                .ok()
                .map(|entry| entry.value.clone());
            let monitors = &mut self.monitoring.monitors;
            let replaced = monitors.iter().position(|monitor| monitor.id == id);
            let replaced = replaced.map(|position| monitors.remove(position));
            let tripped = replaced.is_some_and(|old| old.kind == data.kind && old.tripped);
            monitors.push(Monitor {
                id,
                component: data.component.clone(),
                variable: data.variable.clone(),
                kind: data.kind,
                value: data.value,
                severity: data.severity,
                transaction: data.transaction,
                tripped,
                reference,
            });
        }
        SetMonitoringResult {
            id,
            status_info: None,
            status: status.unwrap_or(SetMonitoringStatus::Accepted),
            kind: data.kind,
            component: data.component,
            variable: data.variable,
            severity: data.severity,
        }
    }

    fn monitor_refusal(&self, data: &SetMonitoringData) -> Option<SetMonitoringStatus> {
        let entry = match self.device_model.entry(&data.component, &data.variable) {
            Ok(entry) => entry,
            Err(Missing::Component) => return Some(SetMonitoringStatus::UnknownComponent),
            Err(Missing::Variable) => return Some(SetMonitoringStatus::UnknownVariable),
        };
        let characteristics = &entry.characteristics;
        let numeric = matches!(
            characteristics.data_type,
            DataType::Integer | DataType::Decimal
        );
        let in_limits = characteristics
            .min_limit
            .is_none_or(|min| data.value >= min)
            && characteristics
                .max_limit
                .is_none_or(|max| data.value <= max);
        let threshold = matches!(
            data.kind,
            MonitorType::UpperThreshold | MonitorType::LowerThreshold
        );
        let periodic = matches!(
            data.kind,
            MonitorType::Periodic | MonitorType::PeriodicClockAligned
        );
        if periodic || (threshold && !numeric) {
            return Some(SetMonitoringStatus::UnsupportedMonitorType);
        }
        let monitors = &self.monitoring.monitors;
        let existing = data
            .id
            .map(|id| monitors.iter().find(|monitor| monitor.id == id));
        let moved = existing.flatten().is_some_and(|monitor| {
            monitor.component != data.component || monitor.variable != data.variable
        });
        let rejected = matches!(existing, Some(None))
            || moved
            || data.severity > HIGHEST_SEVERITY
            || (threshold && !in_limits)
            || (data.kind == MonitorType::Delta && data.value <= 0.0);
        if rejected {
            return Some(SetMonitoringStatus::Rejected);
        }
        let duplicate = monitors.iter().any(|monitor| {
            Some(monitor.id) != data.id
                && monitor.component == data.component
                && monitor.variable == data.variable
                && monitor.kind == data.kind
                && monitor.severity == data.severity
        });
        duplicate.then_some(SetMonitoringStatus::Duplicate)
    }

    fn new_monitor_id(&mut self) -> i32 {
        self.monitoring.next_id += 1;
        self.monitoring.next_id
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
