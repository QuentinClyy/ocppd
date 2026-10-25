use crate::protocol::v16::messages::core::configuration::{
    ChangeConfigurationRequest, ConfigurationStatus, GetConfigurationRequest,
    GetConfigurationResponse, KeyValue,
};
use crate::protocol::v16::messages::datatypes::meter_value::Measurand;
use crate::wire::ci_string::{CiString, TooLong};
use serde_json::Value;

pub const AUTHORIZATION_CACHE_ENABLED: &str = "AuthorizationCacheEnabled";
pub const AUTHORIZATION_KEY: &str = "AuthorizationKey";
pub const CERTIFICATE_STORE_MAX_LENGTH: &str = "CertificateStoreMaxLength";
pub const CHARGE_PROFILE_MAX_STACK_LEVEL: &str = "ChargeProfileMaxStackLevel";
pub const CHARGING_SCHEDULE_MAX_PERIODS: &str = "ChargingScheduleMaxPeriods";
pub const CONNECTION_TIME_OUT: &str = "ConnectionTimeOut";
pub const CPO_NAME: &str = "CpoName";
pub const HEARTBEAT_INTERVAL: &str = "HeartbeatInterval";
pub const LOCAL_AUTH_LIST_ENABLED: &str = "LocalAuthListEnabled";
pub const LOCAL_AUTHORIZE_OFFLINE: &str = "LocalAuthorizeOffline";
pub const LOCAL_AUTH_LIST_MAX_LENGTH: &str = "LocalAuthListMaxLength";
pub const LOCAL_PRE_AUTHORIZE: &str = "LocalPreAuthorize";
pub const MAX_CHARGING_PROFILES_INSTALLED: &str = "MaxChargingProfilesInstalled";
pub const METER_VALUE_SAMPLE_INTERVAL: &str = "MeterValueSampleInterval";
pub const METER_VALUES_SAMPLED_DATA: &str = "MeterValuesSampledData";
pub const SECURITY_PROFILE: &str = "SecurityProfile";
pub const SEND_LOCAL_LIST_MAX_LENGTH: &str = "SendLocalListMaxLength";
pub const TRANSACTION_MESSAGE_ATTEMPTS: &str = "TransactionMessageAttempts";
pub const TRANSACTION_MESSAGE_RETRY_INTERVAL: &str = "TransactionMessageRetryInterval";

const DEFAULT_MEASURANDS: &str =
    "Energy.Active.Import.Register,Power.Active.Import,Current.Import,Current.Offered,Voltage";
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
    SecurityProfile,
    WriteOnly,
    Text,
}

impl Configuration {
    pub fn new(connectors: usize) -> Result<Self, TooLong> {
        let table = [
            (AUTHORIZATION_CACHE_ENABLED, "true", Access::Boolean),
            (AUTHORIZATION_KEY, "", Access::WriteOnly),
            (
                "AuthorizeRemoteTxRequests",
                "false",
                Access::CurrentValueOnly,
            ),
            (CERTIFICATE_STORE_MAX_LENGTH, "10", Access::ReadOnly),
            (CHARGE_PROFILE_MAX_STACK_LEVEL, "10", Access::ReadOnly),
            (
                "ChargingScheduleAllowedChargingRateUnit",
                "Current,Power",
                Access::ReadOnly,
            ),
            (CHARGING_SCHEDULE_MAX_PERIODS, "24", Access::ReadOnly),
            ("ClockAlignedDataInterval", "0", Access::CurrentValueOnly),
            (CONNECTION_TIME_OUT, "60", Access::Integer),
            ("ConnectorPhaseRotation", "1.RST", Access::CurrentValueOnly),
            (CPO_NAME, "", Access::Text),
            ("GetConfigurationMaxKeys", "50", Access::ReadOnly),
            (HEARTBEAT_INTERVAL, "86400", Access::Integer),
            (LOCAL_AUTH_LIST_ENABLED, "true", Access::Boolean),
            (LOCAL_AUTH_LIST_MAX_LENGTH, "100", Access::ReadOnly),
            (LOCAL_AUTHORIZE_OFFLINE, "false", Access::Boolean),
            (LOCAL_PRE_AUTHORIZE, "false", Access::Boolean),
            (MAX_CHARGING_PROFILES_INSTALLED, "20", Access::ReadOnly),
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
            (SECURITY_PROFILE, "0", Access::SecurityProfile),
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
                "Core,FirmwareManagement,LocalAuthListManagement,RemoteTrigger,Reservation,SmartCharging",
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
            Access::SecurityProfile => {
                let current: u32 = key.value.as_str().parse().unwrap_or(0);
                value
                    .parse::<u32>()
                    .is_ok_and(|profile| profile > current && profile <= 3)
            }
            Access::Text => true,
            Access::WriteOnly => hex(value).is_some_and(|key| (16..=20).contains(&key.len())),
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

    pub fn set_text(&mut self, name: &str, value: CiString<500>) {
        if let Some(index) = self.position(name) {
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

    pub fn text(&self, name: &str) -> Option<&str> {
        Some(self.find(name)?.value.as_str())
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
            value: (self.access != Access::WriteOnly).then(|| self.value.clone()),
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

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(text.get(at..at + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
#[path = "configuration.test.rs"]
mod tests;
