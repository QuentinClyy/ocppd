use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::Entry;
use crate::protocol::v201::charging_station::device_model::table::{
    AVAILABILITY_STATE, ITEMS_PER_REPORT,
};
use crate::protocol::v201::messages::datatypes::component::AttributeType;
use crate::protocol::v201::messages::provisioning::report::{
    GenericDeviceModelStatus, GetBaseReportRequest, GetBaseReportResponse, Mutability,
    NotifyReportRequest, ReportBase, ReportData, VariableAttribute,
};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::json;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn get_base_report(
        &mut self,
        now: u64,
        request: GetBaseReportRequest,
    ) -> GetBaseReportResponse {
        self.refresh_device_model(now);
        let base = request.report_base;
        let data: Vec<ReportData> = self
            .device_model
            .entries()
            .iter()
            .filter(|entry| included(base, entry))
            .map(report_data)
            .collect();
        let chunks: Vec<&[ReportData]> = data.chunks(ITEMS_PER_REPORT).collect();
        for (seq_no, chunk) in chunks.iter().enumerate() {
            let report = NotifyReportRequest {
                request_id: request.request_id,
                generated_at: DateTime { unix_ms: now },
                report_data: Some(chunk.to_vec()),
                tbc: seq_no + 1 < chunks.len(),
                seq_no: seq_no as u32,
            };
            self.outbox.push("NotifyReport", 0, json!(report));
        }
        let status = match chunks.is_empty() {
            true => GenericDeviceModelStatus::EmptyResultSet,
            false => GenericDeviceModelStatus::Accepted,
        };
        GetBaseReportResponse {
            status,
            status_info: None,
        }
    }
}

fn included(base: ReportBase, entry: &Entry) -> bool {
    match base {
        ReportBase::FullInventory => true,
        ReportBase::ConfigurationInventory => entry.mutability != Mutability::ReadOnly,
        ReportBase::SummaryInventory => entry.variable.name.as_str() == AVAILABILITY_STATE,
    }
}

fn report_data(entry: &Entry) -> ReportData {
    let readable = entry.mutability != Mutability::WriteOnly;
    let value = readable
        .then_some(&entry.value)
        .and_then(|value| CiString::new(value).ok());
    ReportData {
        component: entry.component.clone(),
        variable: entry.variable.clone(),
        variable_attribute: vec![VariableAttribute {
            kind: AttributeType::Actual,
            value,
            mutability: entry.mutability,
            persistent: entry.mutability != Mutability::ReadOnly,
            constant: false,
        }],
        variable_characteristics: Some(entry.characteristics.clone()),
    }
}

#[cfg(test)]
#[path = "report.test.rs"]
mod tests;
