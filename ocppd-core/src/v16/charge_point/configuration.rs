use crate::v16::messages::core::configuration::{
    ChangeConfigurationRequest, ConfigurationStatus, GetConfigurationRequest,
    GetConfigurationResponse, KeyValue,
};
use crate::v16::messages::datatypes::meter_value::Measurand;
use crate::wire::ci_string::{CiString, TooLong};
use serde_json::Value;

pub const CONNECTION_TIME_OUT: &str = "ConnectionTimeOut";
pub const HEARTBEAT_INTERVAL: &str = "HeartbeatInterval";
pub const LOCAL_AUTH_LIST_ENABLED: &str = "LocalAuthListEnabled";
pub const LOCAL_AUTH_LIST_MAX_LENGTH: &str = "LocalAuthListMaxLength";
pub const LOCAL_PRE_AUTHORIZE: &str = "LocalPreAuthorize";
pub const METER_VALUE_SAMPLE_INTERVAL: &str = "MeterValueSampleInterval";
pub const METER_VALUES_SAMPLED_DATA: &str = "MeterValuesSampledData";
pub const SEND_LOCAL_LIST_MAX_LENGTH: &str = "SendLocalListMaxLength";
pub const TRANSACTION_MESSAGE_ATTEMPTS: &str = "TransactionMessageAttempts";
pub const TRANSACTION_MESSAGE_RETRY_INTERVAL: &str = "TransactionMessageRetryInterval";

const DEFAULT_MEASURANDS: &str =
    "Energy.Active.Import.Register,Power.Active.Import,Current.Import,Voltage";
const SUPPORTED_MEASURANDS: [Measurand; 7] = [
    Measurand::EnergyActiveImportRegister,
    Measurand::EnergyActiveImportInterval,
    Measurand::PowerActiveImport,
    Measurand::PowerOffered,
    Measurand::CurrentImport,
    Measurand::CurrentOffered,
    Measurand::Voltage,
];

pub struct Configuration {
    keys: Vec<Key>,
}

struct Key {
    name: CiString<50>,
    value: CiString<500>,
    access: Access,
}

#[derive(Clone, Copy, PartialEq)]
enum Access {
    ReadOnly,
    Integer,
    Boolean,
    Measurands,
    CurrentValueOnly,
}

impl Configuration {
    pub fn new(connectors: usize) -> Result<Self, TooLong> {
        let table = [
            (
                "AuthorizeRemoteTxRequests",
                "false",
                Access::CurrentValueOnly,
            ),
            ("ClockAlignedDataInterval", "0", Access::CurrentValueOnly),
            (CONNECTION_TIME_OUT, "60", Access::Integer),
            ("ConnectorPhaseRotation", "1.RST", Access::CurrentValueOnly),
            ("GetConfigurationMaxKeys", "50", Access::ReadOnly),
            (HEARTBEAT_INTERVAL, "86400", Access::Integer),
            (LOCAL_AUTH_LIST_ENABLED, "true", Access::Boolean),
            (LOCAL_AUTH_LIST_MAX_LENGTH, "100", Access::ReadOnly),
            ("LocalAuthorizeOffline", "false", Access::CurrentValueOnly),
            (LOCAL_PRE_AUTHORIZE, "false", Access::Boolean),
            ("MeterValuesAlignedData", "", Access::CurrentValueOnly),
            (
                METER_VALUES_SAMPLED_DATA,
                DEFAULT_MEASURANDS,
                Access::Measurands,
            ),
            ("MeterValuesSampledDataMaxLength", "7", Access::ReadOnly),
            (METER_VALUE_SAMPLE_INTERVAL, "60", Access::Integer),
            (
                "NumberOfConnectors",
                &connectors.to_string(),
                Access::ReadOnly,
            ),
            ("ReserveConnectorZeroSupported", "false", Access::ReadOnly),
            ("ResetRetries", "0", Access::Integer),
            (SEND_LOCAL_LIST_MAX_LENGTH, "100", Access::ReadOnly),
            (
                "StopTransactionOnEVSideDisconnect",
                "true",
                Access::CurrentValueOnly,
            ),
            (
                "StopTransactionOnInvalidId",
                "true",
                Access::CurrentValueOnly,
            ),
            ("StopTxnAlignedData", "", Access::CurrentValueOnly),
            ("StopTxnSampledData", "", Access::CurrentValueOnly),
            (
                "SupportedFeatureProfiles",
                "Core,LocalAuthListManagement,RemoteTrigger,Reservation",
                Access::ReadOnly,
            ),
            (TRANSACTION_MESSAGE_ATTEMPTS, "3", Access::Integer),
            (TRANSACTION_MESSAGE_RETRY_INTERVAL, "60", Access::Integer),
            (
                "UnlockConnectorOnEVSideDisconnect",
                "true",
                Access::CurrentValueOnly,
            ),
        ];
        let mut keys = Vec::with_capacity(table.len());
        for (name, value, access) in table {
            keys.push(Key {
                name: CiString::new(name)?,
                value: CiString::new(value)?,
                access,
            });
        }
        Ok(Configuration { keys })
    }

    pub fn get(&self, request: &GetConfigurationRequest) -> GetConfigurationResponse {
        let requested = request.key.as_deref().unwrap_or_default();
        if requested.is_empty() {
            let all = self.keys.iter().map(Key::key_value).collect();
            return GetConfigurationResponse {
                configuration_key: Some(all),
                unknown_key: None,
            };
        }
        let mut known = Vec::new();
        let mut unknown = Vec::new();
        for name in requested {
            match self.find(name.as_str()) {
                Some(key) => known.push(key.key_value()),
                None => unknown.push(name.clone()),
            }
        }
        GetConfigurationResponse {
            configuration_key: Some(known).filter(|known| !known.is_empty()),
            unknown_key: Some(unknown).filter(|unknown| !unknown.is_empty()),
        }
    }

    pub fn change(&mut self, request: &ChangeConfigurationRequest) -> ConfigurationStatus {
        let Some(index) = self.position(request.key.as_str()) else {
            return ConfigurationStatus::NotSupported;
        };
        let key = &mut self.keys[index];
        let value = request.value.as_str();
        let valid = match key.access {
            Access::ReadOnly => false,
            Access::Integer => value.parse::<u32>().is_ok(),
            Access::Boolean => value == "true" || value == "false",
            Access::Measurands => measurands(value).is_some(),
            Access::CurrentValueOnly => value == key.value.as_str(),
        };
        if !valid {
            return ConfigurationStatus::Rejected;
        }
        key.value = request.value.clone();
        ConfigurationStatus::Accepted
    }

    pub fn set_seconds(&mut self, name: &str, seconds: u32) {
        let Some(index) = self.position(name) else {
            return;
        };
        if let Ok(value) = CiString::new(&seconds.to_string()) {
            self.keys[index].value = value;
        }
    }

    pub fn interval_ms(&self, name: &str) -> Option<u64> {
        let seconds: u64 = self.find(name)?.value.as_str().parse().ok()?;
        Some(seconds * 1000).filter(|&ms| ms > 0)
    }

    pub fn flag(&self, name: &str) -> bool {
        self.find(name)
            .is_some_and(|key| key.value.as_str() == "true")
    }

    pub fn number(&self, name: &str) -> Option<u32> {
        self.find(name)?.value.as_str().parse().ok()
    }

    pub fn measurands(&self) -> Vec<Measurand> {
        self.find(METER_VALUES_SAMPLED_DATA)
            .and_then(|key| measurands(key.value.as_str()))
            .unwrap_or_default()
    }

    fn position(&self, name: &str) -> Option<usize> {
        self.keys
            .iter()
            .position(|key| key.name.as_str().eq_ignore_ascii_case(name))
    }

    fn find(&self, name: &str) -> Option<&Key> {
        self.position(name).map(|index| &self.keys[index])
    }
}

impl Key {
    fn key_value(&self) -> KeyValue {
        KeyValue {
            key: self.name.clone(),
            readonly: self.access == Access::ReadOnly,
            value: Some(self.value.clone()),
        }
    }
}

fn measurands(list: &str) -> Option<Vec<Measurand>> {
    let mut parsed = Vec::new();
    for item in list
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let measurand: Measurand = serde_json::from_value(Value::String(item.to_owned())).ok()?;
        if !SUPPORTED_MEASURANDS.contains(&measurand) || parsed.contains(&measurand) {
            return None;
        }
        parsed.push(measurand);
    }
    Some(parsed)
}

#[cfg(test)]
#[path = "configuration.test.rs"]
mod tests;
