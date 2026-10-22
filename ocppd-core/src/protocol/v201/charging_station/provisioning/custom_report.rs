use super::report::report_data;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::Entry;
use crate::protocol::v201::messages::datatypes::component::{Component, ComponentVariable};
use crate::protocol::v201::messages::provisioning::report::{
    ComponentCriterion, GetReportRequest, GetReportResponse, ReportData,
};

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn get_report(
        &mut self,
        now: u64,
        request: GetReportRequest,
    ) -> GetReportResponse {
        self.refresh_device_model(now);
        let entries = self.device_model.entries();
        let wanted = |entry: &&Entry| {
            let variables = request.component_variable.as_ref();
            let criteria = request.component_criteria.as_ref();
            variables.is_none_or(|variables| variables.iter().any(|wanted| selects(wanted, entry)))
                && criteria.is_none_or(|criteria| {
                    let mut criteria = criteria.iter();
                    criteria.any(|&criterion| meets(entries, &entry.component, criterion))
                })
        };
        let data: Vec<ReportData> = entries.iter().filter(wanted).map(report_data).collect();
        GetReportResponse {
            status: self.send_report(now, request.request_id, data),
            status_info: None,
        }
    }
}

fn selects(wanted: &ComponentVariable, entry: &Entry) -> bool {
    let (component, variable) = (&wanted.component, &entry.variable);
    let evse = match (component.evse, entry.component.evse) {
        (None, _) => true,
        (Some(wanted), Some(evse)) => {
            wanted.id == evse.id
                && wanted
                    .connector_id
                    .is_none_or(|connector| Some(connector) == evse.connector_id)
        }
        (Some(_), None) => false,
    };
    let instance = match (&component.instance, &entry.component.instance) {
        (None, _) => true,
        (Some(wanted), Some(instance)) => wanted.matches(instance),
        (Some(_), None) => false,
    };
    let variable = wanted.variable.as_ref().is_none_or(|wanted| {
        wanted.name.matches(&variable.name)
            && match (&wanted.instance, &variable.instance) {
                (None, _) => true,
                (Some(wanted), Some(instance)) => wanted.matches(instance),
                (Some(_), None) => false,
            }
    });
    component.name.matches(&entry.component.name) && evse && instance && variable
}

fn meets(entries: &[Entry], component: &Component, criterion: ComponentCriterion) -> bool {
    let flag = |name: &str| {
        let mut entries = entries.iter();
        let found = entries.find(|entry| {
            entry.component == *component
                && entry.variable.name.as_str() == name
                && entry.variable.instance.is_none()
        });
        found.map(|entry| entry.value == "true")
    };
    match criterion {
        ComponentCriterion::Active => flag("Active") != Some(false),
        ComponentCriterion::Available => flag("Available") != Some(false),
        ComponentCriterion::Enabled => flag("Enabled") != Some(false),
        ComponentCriterion::Problem => flag("Problem") == Some(true),
    }
}

#[cfg(test)]
#[path = "custom_report.test.rs"]
mod tests;
