use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Rejection {
    #[serde(skip_serializing_if = "Option::is_none")]
    station: Option<String>,
    at: u64,
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    reason: String,
}

impl Rejection {
    pub fn new(
        command: Option<&str>,
        station: Option<&str>,
        reason: impl Into<String>,
    ) -> Rejection {
        Rejection {
            station: station.map(str::to_owned),
            at: crate::shell::now_ms(),
            kind: "command_rejected",
            command: command.map(str::to_owned),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
#[path = "rejection.test.rs"]
mod tests;
