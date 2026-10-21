use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Violation {
    Formation,
    Occurrence,
    Type,
    Property,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Invalid {
    pub violation: Violation,
    pub reason: String,
}

pub fn decode<T: DeserializeOwned + Serialize>(payload: Value) -> Result<T, Invalid> {
    if !payload.is_object() {
        return Err(invalid(Violation::Formation, "payload is not an object"));
    }
    let decoded: T = serde_json::from_value(payload.clone()).map_err(|err| {
        let reason = err.to_string();
        invalid(classify(&reason), &reason)
    })?;
    let known = serde_json::to_value(&decoded)
        .map_err(|err| invalid(Violation::Formation, &err.to_string()))?;
    match extra(&payload, &known, "") {
        Some(found) => Err(found),
        None => Ok(decoded),
    }
}

fn classify(reason: &str) -> Violation {
    if reason.starts_with("missing field") {
        Violation::Occurrence
    } else if reason.starts_with("invalid type") {
        Violation::Type
    } else {
        Violation::Property
    }
}

fn extra(payload: &Value, known: &Value, path: &str) -> Option<Invalid> {
    match (payload, known) {
        (Value::Object(payload), Value::Object(known)) => extra_keys(payload, known, path),
        (Value::Array(payload), Value::Array(known)) => payload
            .iter()
            .zip(known)
            .enumerate()
            .find_map(|(index, (item, known))| extra(item, known, &format!("{path}[{index}]"))),
        _ => None,
    }
}

fn extra_keys(
    payload: &Map<String, Value>,
    known: &Map<String, Value>,
    path: &str,
) -> Option<Invalid> {
    payload.iter().find_map(|(key, value)| {
        let field = match path {
            "" => key.clone(),
            _ => format!("{path}.{key}"),
        };
        match known.get(key) {
            Some(known) => extra(value, known, &field),
            None if value.is_null() => Some(invalid(Violation::Type, &format!("{field} is null"))),
            None => Some(invalid(
                Violation::Formation,
                &format!("unknown field {field}"),
            )),
        }
    })
}

fn invalid(violation: Violation, reason: &str) -> Invalid {
    Invalid {
        violation,
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
#[path = "payload.test.rs"]
mod tests;
