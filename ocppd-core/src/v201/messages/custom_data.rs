use serde_json::Value;

pub fn without_custom_data(value: Value) -> Value {
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .filter(|(key, _)| key != "customData")
                .map(|(key, value)| (key, without_custom_data(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(without_custom_data).collect()),
        other => other,
    }
}

#[cfg(test)]
#[path = "custom_data.test.rs"]
mod tests;
