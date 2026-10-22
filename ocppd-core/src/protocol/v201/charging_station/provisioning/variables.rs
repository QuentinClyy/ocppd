use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::shortcut;
use crate::protocol::v201::charging_station::device_model::table::{
    AVAILABILITY_STATE, CLOCK_CTRLR, DATE_TIME,
};
use crate::protocol::v201::charging_station::local_auth::{
    AUTH_CACHE_CTRLR, ENTRIES, LOCAL_AUTH_LIST_CTRLR,
};
use crate::protocol::v201::charging_station::security::certificates::ADDITIONAL_ROOT_CERTIFICATE_CHECK;
use crate::protocol::v201::charging_station::security::{
    BASIC_AUTH_PASSWORD, CERTIFICATE_ENTRIES, RECONFIGURATION_OF_SECURITY_PARAMETERS,
    SECURITY_CTRLR, SECURITY_PROFILE,
};
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::availability::ConnectorStatus;
use crate::protocol::v201::messages::datatypes::component::{AttributeType, Evse};
use crate::protocol::v201::messages::provisioning::variables::{
    GetVariableResult, GetVariableStatus, GetVariablesRequest, GetVariablesResponse,
    SetVariableResult, SetVariableStatus, SetVariablesRequest, SetVariablesResponse,
};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;

const PROVISIONED: [&str; 2] = [SECURITY_PROFILE, ADDITIONAL_ROOT_CERTIFICATE_CHECK];

impl ChargingStation {
    pub fn set_basic_auth_password(&mut self, password: &str) -> bool {
        self.configure("SecurityCtrlr.BasicAuthPassword", password)
    }

    pub fn configure(&mut self, key: &str, value: &str) -> bool {
        let Some((component, variable)) = shortcut::parse(key) else {
            return false;
        };
        let provisioned = component.name.as_str() == SECURITY_CTRLR
            && PROVISIONED.contains(&variable.name.as_str())
            && variable.instance.is_none();
        match provisioned {
            true => self.device_model.provision(&component, &variable, value),
            false => {
                self.device_model.set(&component, &variable, value) == SetVariableStatus::Accepted
            }
        }
    }

    pub(in crate::protocol::v201::charging_station) fn get_variables(
        &mut self,
        now: u64,
        request: GetVariablesRequest,
    ) -> GetVariablesResponse {
        self.refresh_device_model(now);
        let results = request.get_variable_data.into_iter().map(|data| {
            let attribute_type = data.attribute_type.unwrap_or_default();
            let (attribute_status, value) = match attribute_type {
                AttributeType::Actual => self.device_model.get(&data.component, &data.variable),
                _ => (GetVariableStatus::NotSupportedAttributeType, None),
            };
            GetVariableResult {
                attribute_status_info: None,
                attribute_status,
                attribute_type,
                attribute_value: value.and_then(|value| CiString::new(&value).ok()),
                component: data.component,
                variable: data.variable,
            }
        });
        GetVariablesResponse {
            get_variable_result: results.collect(),
        }
    }

    pub(in crate::protocol::v201::charging_station) fn set_variables(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: SetVariablesRequest,
    ) -> SetVariablesResponse {
        let mut results = Vec::new();
        for data in request.set_variable_data {
            let attribute_type = data.attribute_type.unwrap_or_default();
            let value = data.attribute_value.as_str();
            let attribute_status = match attribute_type {
                AttributeType::Actual => {
                    self.device_model
                        .set(&data.component, &data.variable, value)
                }
                _ => SetVariableStatus::NotSupportedAttributeType,
            };
            let secret = data.variable.name.as_str() == BASIC_AUTH_PASSWORD;
            if attribute_status == SetVariableStatus::Accepted && secret {
                self.security_event(out, now, RECONFIGURATION_OF_SECURITY_PARAMETERS);
            } else if attribute_status == SetVariableStatus::Accepted {
                let kind = EventKind::VariableChanged {
                    component: data.component.name.as_str().to_owned(),
                    variable: data.variable.name.as_str().to_owned(),
                    value: value.to_owned(),
                };
                self.emit(out, now, kind);
            }
            results.push(SetVariableResult {
                attribute_type,
                attribute_status,
                attribute_status_info: None,
                component: data.component,
                variable: data.variable,
            });
        }
        SetVariablesResponse {
            set_variable_result: results,
        }
    }

    pub(in crate::protocol::v201::charging_station) fn refresh_device_model(&mut self, now: u64) {
        let date_time = DateTime { unix_ms: now }.to_string();
        self.device_model
            .set_value(CLOCK_CTRLR, None, DATE_TIME, date_time);
        let certificates = self.certificate_count().to_string();
        self.device_model
            .set_value(SECURITY_CTRLR, None, CERTIFICATE_ENTRIES, certificates);
        let entries = self.local_list.tokens.len().to_string();
        self.device_model
            .set_value(LOCAL_AUTH_LIST_CTRLR, None, ENTRIES, entries);
        let entries = self.cache.tokens.len().to_string();
        self.device_model
            .set_value(AUTH_CACHE_CTRLR, None, ENTRIES, entries);
        let station = match self.station_availability.operative {
            true => "Available",
            false => "Unavailable",
        };
        self.device_model.set_value(
            "ChargingStation",
            None,
            AVAILABILITY_STATE,
            station.to_owned(),
        );
        for index in 0..self.station.connectors.len() {
            let state = format!("{:?}", self.connector_status(index));
            let id = index as u32 + 1;
            for (component, connector_id) in [("EVSE", None), ("Connector", Some(1))] {
                let evse = Some(Evse { id, connector_id });
                self.device_model
                    .set_value(component, evse, AVAILABILITY_STATE, state.clone());
            }
        }
    }

    pub(in crate::protocol::v201::charging_station) fn connector_status(
        &self,
        index: usize,
    ) -> ConnectorStatus {
        if !self.operative(index) {
            return ConnectorStatus::Unavailable;
        }
        match self.station.connectors[index].plugged {
            Some(_) => ConnectorStatus::Occupied,
            None => ConnectorStatus::Available,
        }
    }
}
