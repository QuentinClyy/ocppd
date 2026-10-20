use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn round_trip<T: Serialize + DeserializeOwned>(value: &Value) -> Value {
    let typed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|err| panic!("decoding {value}: {err}"));
    serde_json::to_value(&typed).unwrap()
}
