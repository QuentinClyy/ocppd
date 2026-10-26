use crate::identity::Identities;
use crate::model::Model;
use crate::protocol::{Protocol, Version};
use crate::rejection::Rejection;
use crate::shell;
use ocppd_core::api::event::Event;
use ocppd_core::api::intent::Intent;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;
use tokio::time::{Instant, timeout_at};

const STOP_WITHIN: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
pub struct Spawn {
    #[serde(flatten)]
    identities: Identities,
    #[serde(default = "jitter_by_default")]
    jitter: bool,
    #[serde(default)]
    configuration: BTreeMap<String, String>,
    #[serde(default)]
    ocpp: Version,
    csms: Option<String>,
    #[serde(flatten)]
    model: Model,
}

fn jitter_by_default() -> bool {
    true
}

fn refusal_hint(key: &str) -> &'static str {
    match key {
        "AuthorizationKey" => {
            ": needs 16-20 bytes as hex; use \"password\" for a plain-text password"
        }
        _ => "",
    }
}

#[derive(Deserialize)]
pub struct Routed {
    #[serde(skip)]
    name: String,
    station: String,
    #[serde(flatten)]
    intent: Intent,
}

#[derive(Deserialize)]
pub struct Remove {
    pub station: String,
}

pub enum Command {
    Shutdown,
    Spawn(Spawn),
    Remove(Remove),
    Routed(Routed),
}

pub struct Running {
    intents: UnboundedSender<Intent>,
    task: JoinHandle<()>,
}

pub type Stations = HashMap<String, Running>;

pub fn parse(line: &str) -> Result<Command, Rejection> {
    let value: Value =
        serde_json::from_str(line).map_err(|err| Rejection::new(None, None, err.to_string()))?;
    let name = value["command"].as_str().map(str::to_owned);
    let station = value["station"].as_str().map(str::to_owned);
    let command = match name.as_deref() {
        Some("shutdown") => Ok(Command::Shutdown),
        Some("spawn") => serde_json::from_value(value).map(Command::Spawn),
        Some("remove") => serde_json::from_value(value).map(Command::Remove),
        _ => serde_json::from_value(value).map(|routed: Routed| {
            Command::Routed(Routed {
                name: name.clone().unwrap_or_default(),
                ..routed
            })
        }),
    };
    command.map_err(|err| Rejection::new(name.as_deref(), station.as_deref(), err.to_string()))
}

pub fn remove(command: &Remove, stations: &mut Stations) -> Result<(), Rejection> {
    let running = stations
        .remove(&command.station)
        .ok_or_else(|| Rejection::new(Some("remove"), Some(&command.station), "unknown station"))?;
    running.task.abort();
    Ok(())
}

pub fn route(command: Routed, stations: &Stations) -> Result<(), Rejection> {
    let reject = |reason| Rejection::new(Some(&command.name), Some(&command.station), reason);
    let Some(Running { intents, .. }) = stations.get(&command.station) else {
        return Err(reject("unknown station"));
    };
    intents
        .send(command.intent)
        .map_err(|_| reject("station stopped"))
}

pub fn spawn(
    csms: &str,
    command: &Spawn,
    events: &UnboundedSender<Event>,
    stations: &mut Stations,
) -> Vec<Rejection> {
    let identities = match command.identities.list() {
        Ok(identities) => identities,
        Err(err) => return vec![Rejection::new(Some("spawn"), None, err)],
    };
    let mut rejections = Vec::new();
    for identity in identities {
        let id = identity.id;
        let reject = |reason: String| Rejection::new(Some("spawn"), Some(&id), reason);
        if stations.contains_key(&id) {
            rejections.push(reject("already exists".into()));
            continue;
        }
        let base = command
            .csms
            .as_deref()
            .unwrap_or(csms)
            .trim_end_matches('/');
        let url = format!("{base}/{id}");
        let station = match command.model.station(&id, command.jitter) {
            Ok(station) => station,
            Err(err) => {
                rejections.push(reject(err.into()));
                continue;
            }
        };
        match Protocol::new(command.ocpp, station) {
            Ok(mut protocol) => {
                let refused = command
                    .configuration
                    .iter()
                    .find(|(key, value)| !protocol.configure(key, value));
                if let Some((key, _)) = refused {
                    rejections.push(reject(format!("{key} refused{}", refusal_hint(key))));
                    continue;
                }
                let password = identity.password.as_deref();
                if password.is_some_and(|password| !protocol.set_basic_auth_password(password)) {
                    rejections.push(reject("password refused".into()));
                    continue;
                }
                if !protocol.set_csms_url(base) {
                    rejections.push(reject("csms url too long".into()));
                    continue;
                }
                let (intents, receiver) = unbounded_channel();
                let task = tokio::spawn(shell::run(protocol, url, events.clone(), receiver));
                stations.insert(id, Running { intents, task });
            }
            Err(err) => rejections.push(reject(err.to_string())),
        }
    }
    rejections
}

pub async fn stop(stations: Stations) {
    let deadline = Instant::now() + STOP_WITHIN;
    let mut tasks = Vec::new();
    for running in stations.into_values() {
        let _ = running.intents.send(Intent::Shutdown);
        tasks.push(running.task);
    }
    for mut task in tasks {
        if timeout_at(deadline, &mut task).await.is_err() {
            task.abort();
        }
    }
}
