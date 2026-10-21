use crate::fixtures::v201::station::Run;
use serde_json::Value;

pub fn reports(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "NotifyReport")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn variables(reports: &[Value]) -> Vec<String> {
    reports
        .iter()
        .flat_map(|report| report["reportData"].as_array().unwrap().clone())
        .map(|data| {
            format!(
                "{}.{}",
                data["component"]["name"].as_str().unwrap(),
                data["variable"]["name"].as_str().unwrap()
            )
        })
        .collect()
}
