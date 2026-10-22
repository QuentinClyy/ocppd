mod alert;
mod hard_wired;
mod report;

use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::Missing;
use crate::protocol::v201::messages::datatypes::component::{Component, Variable};
use crate::protocol::v201::messages::datatypes::generic_status::GenericStatus;
use crate::protocol::v201::messages::diagnostics::monitoring::{
    ClearMonitoringResult, ClearMonitoringStatus, ClearVariableMonitoringRequest,
    ClearVariableMonitoringResponse, MonitorType, MonitoringBase, SetMonitoringBaseRequest,
    SetMonitoringBaseResponse, SetMonitoringData, SetMonitoringLevelRequest,
    SetMonitoringLevelResponse, SetMonitoringResult, SetMonitoringStatus,
    SetVariableMonitoringRequest, SetVariableMonitoringResponse,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, GenericDeviceModelStatus};

pub use hard_wired::{LOCK, PROBLEM};

pub const MONITORING_CTRLR: &str = "MonitoringCtrlr";
pub const OFFLINE_QUEUING_SEVERITY: &str = "OfflineQueuingSeverity";
pub const ACTIVE_MONITORING_LEVEL: &str = "ActiveMonitoringLevel";
pub const ACTIVE_MONITORING_BASE: &str = "ActiveMonitoringBase";
pub const MONITORING_BASES: &str = "All,FactoryDefault,HardWiredOnly";
pub const ITEMS_PER_MONITORING: usize = 50;
pub const HIGHEST_SEVERITY: u32 = 9;
const SHORTEST_PERIOD_S: f64 = 1.0;

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
    next_at: Option<u64>,
}

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn set_variable_monitoring(
        &mut self,
        now: u64,
        request: SetVariableMonitoringRequest,
    ) -> SetVariableMonitoringResponse {
        self.refresh_device_model(now);
        let results = request.set_monitoring_data.into_iter();
        let results = results.map(|data| self.set_monitor(now, data)).collect();
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

    pub(in crate::protocol::v201::charging_station) fn set_monitoring_level(
        &mut self,
        request: SetMonitoringLevelRequest,
    ) -> SetMonitoringLevelResponse {
        let severity = u32::try_from(request.severity).ok();
        let status = match severity.is_some_and(|severity| severity <= HIGHEST_SEVERITY) {
            true => GenericStatus::Accepted,
            false => GenericStatus::Rejected,
        };
        if status == GenericStatus::Accepted {
            let severity = request.severity.to_string();
            self.device_model
                .set_value(MONITORING_CTRLR, None, ACTIVE_MONITORING_LEVEL, severity);
        }
        SetMonitoringLevelResponse {
            status,
            status_info: None,
        }
    }

    pub(in crate::protocol::v201::charging_station) fn set_monitoring_base(
        &mut self,
        request: SetMonitoringBaseRequest,
    ) -> SetMonitoringBaseResponse {
        let base = match request.monitoring_base {
            MonitoringBase::All => "All",
            MonitoringBase::FactoryDefault => "FactoryDefault",
            MonitoringBase::HardWiredOnly => "HardWiredOnly",
        };
        if request.monitoring_base != MonitoringBase::All {
            self.monitoring.monitors.clear();
        }
        self.device_model.set_value(
            MONITORING_CTRLR,
            None,
            ACTIVE_MONITORING_BASE,
            base.to_owned(),
        );
        SetMonitoringBaseResponse {
            status: GenericDeviceModelStatus::Accepted,
            status_info: None,
        }
    }

    pub(in crate::protocol::v201::charging_station) fn monitoring_deadline(&self) -> Option<u64> {
        let monitors = self.monitoring.monitors.iter();
        monitors.filter_map(|monitor| monitor.next_at).min()
    }

    fn set_monitor(&mut self, now: u64, data: SetMonitoringData) -> SetMonitoringResult {
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
                next_at: next_period(data.kind, data.value, now),
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
        if threshold && !numeric {
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
            || (data.kind == MonitorType::Delta && data.value <= 0.0)
            || (periodic && data.value < SHORTEST_PERIOD_S);
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

pub(super) fn next_period(kind: MonitorType, seconds: f64, now: u64) -> Option<u64> {
    let interval = (seconds * 1000.0) as u64;
    match kind {
        MonitorType::Periodic => Some(now + interval),
        MonitorType::PeriodicClockAligned => Some((now / interval + 1) * interval),
        _ => None,
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
