use super::Monitor;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::table::ITEMS_PER_REPORT;
use crate::protocol::v201::charging_station::provisioning::custom_report::selects;
use crate::protocol::v201::messages::diagnostics::monitoring::MonitorType;
use crate::protocol::v201::messages::diagnostics::monitoring_report::{
    GetMonitoringReportRequest, GetMonitoringReportResponse, MonitoringCriterion, MonitoringData,
    NotifyMonitoringReportRequest, VariableMonitoring,
};
use crate::protocol::v201::messages::provisioning::report::GenericDeviceModelStatus;
use crate::wire::date_time::DateTime;
use serde_json::json;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn get_monitoring_report(
        &mut self,
        now: u64,
        request: GetMonitoringReportRequest,
    ) -> GetMonitoringReportResponse {
        let wanted = |monitor: &&Monitor| {
            let variables = request.component_variable.as_ref();
            let criteria = request.monitoring_criteria.as_ref();
            variables.is_none_or(|variables| {
                let mut variables = variables.iter();
                variables.any(|wanted| selects(wanted, &monitor.component, &monitor.variable))
            }) && criteria.is_none_or(|criteria| criteria.contains(&criterion(monitor.kind)))
        };
        let mut data: Vec<MonitoringData> = Vec::new();
        for monitor in self.monitoring.monitors.iter().filter(wanted) {
            let setting = VariableMonitoring {
                id: monitor.id,
                transaction: monitor.transaction,
                value: monitor.value,
                kind: monitor.kind,
                severity: monitor.severity,
            };
            let same = data.iter_mut().find(|data| {
                data.component == monitor.component && data.variable == monitor.variable
            });
            match same {
                Some(same) => same.variable_monitoring.push(setting),
                None => data.push(MonitoringData {
                    component: monitor.component.clone(),
                    variable: monitor.variable.clone(),
                    variable_monitoring: vec![setting],
                }),
            }
        }
        let chunks: Vec<&[MonitoringData]> = data.chunks(ITEMS_PER_REPORT).collect();
        for (seq_no, chunk) in chunks.iter().enumerate() {
            let report = NotifyMonitoringReportRequest {
                monitor: Some(chunk.to_vec()),
                request_id: request.request_id,
                tbc: seq_no + 1 < chunks.len(),
                seq_no: seq_no as u32,
                generated_at: DateTime { unix_ms: now },
            };
            self.outbox.push("NotifyMonitoringReport", 0, json!(report));
        }
        let status = match chunks.is_empty() {
            true => GenericDeviceModelStatus::EmptyResultSet,
            false => GenericDeviceModelStatus::Accepted,
        };
        GetMonitoringReportResponse {
            status,
            status_info: None,
        }
    }
}

fn criterion(kind: MonitorType) -> MonitoringCriterion {
    match kind {
        MonitorType::UpperThreshold | MonitorType::LowerThreshold => {
            MonitoringCriterion::ThresholdMonitoring
        }
        MonitorType::Delta => MonitoringCriterion::DeltaMonitoring,
        MonitorType::Periodic | MonitorType::PeriodicClockAligned => {
            MonitoringCriterion::PeriodicMonitoring
        }
    }
}

#[cfg(test)]
#[path = "report.test.rs"]
mod tests;
