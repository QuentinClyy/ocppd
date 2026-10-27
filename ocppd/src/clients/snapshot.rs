use serde_json::Value;
use std::collections::HashMap;

type Key = (String, String, u64);

#[derive(Default)]
pub struct Snapshot {
    latest: HashMap<Key, (u64, u64, String)>,
    arrivals: u64,
}

impl Snapshot {
    pub fn apply(&mut self, line: &str) {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            return;
        };
        let Some(station) = event["station"].as_str() else {
            return;
        };
        let kind = event["kind"].as_str().unwrap_or_default();
        let slot = event["evse"]
            .as_u64()
            .or(event["connector"].as_u64())
            .unwrap_or(0);
        let at = event["at"].as_u64().unwrap_or(0);
        self.arrivals += 1;
        let key = (station.to_owned(), kind.to_owned(), slot);
        self.latest
            .insert(key, (at, self.arrivals, line.to_owned()));
    }

    pub fn forget(&mut self, station: &str) {
        self.latest.retain(|(id, _, _), _| id != station);
    }

    pub fn lines(&self) -> Vec<String> {
        let mut events: Vec<_> = self.latest.values().collect();
        events.sort_by_key(|(at, arrival, _)| (*at, *arrival));
        events
            .into_iter()
            .map(|(_, _, line)| line.clone())
            .collect()
    }
}

#[cfg(test)]
#[path = "snapshot.test.rs"]
mod tests;
