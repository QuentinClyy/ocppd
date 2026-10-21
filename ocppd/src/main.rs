mod basic_auth;
mod shell;
mod tls;

use ocppd_core::api::event::Event;
use ocppd_core::api::intent::Intent;
use ocppd_core::station::Station;
use ocppd_core::station::connector::Connector;
use ocppd_core::v16::charge_point::ChargePoint;
use ocppd_core::v16::messages::core::configuration::ConfigurationStatus;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::process::ExitCode;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Deserialize)]
struct Spawn {
    prefix: String,
    count: u32,
    #[serde(default = "jitter_by_default")]
    jitter: bool,
    #[serde(default)]
    configuration: BTreeMap<String, String>,
}

fn jitter_by_default() -> bool {
    true
}

#[derive(Deserialize)]
struct Routed {
    station: String,
    #[serde(flatten)]
    intent: Intent,
}

enum Command {
    Spawn(Spawn),
    Routed(Routed),
}

type Stations = HashMap<String, UnboundedSender<Intent>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let Some(csms) = csms_url() else {
        eprintln!("usage: ocppd --csms <url>");
        return ExitCode::from(2);
    };
    let (events, receiver) = unbounded_channel();
    let writer = tokio::spawn(write_events(receiver));
    read_commands(&csms, events).await;
    match writer.await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("event writer failed: {err}");
            ExitCode::FAILURE
        }
    }
}

fn csms_url() -> Option<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag, url] if flag == "--csms" => Some(url.trim_end_matches('/').to_owned()),
        _ => None,
    }
}

async fn read_commands(csms: &str, events: UnboundedSender<Event>) {
    let mut stations = Stations::new();
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => return,
            Err(err) => {
                eprintln!("stdin: {err}");
                return;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        match parse(&line) {
            Ok(Command::Spawn(command)) => spawn(csms, &command, &events, &mut stations),
            Ok(Command::Routed(command)) => route(command, &stations),
            Err(err) => eprintln!("bad command: {err}"),
        }
    }
}

fn parse(line: &str) -> Result<Command, serde_json::Error> {
    let value: Value = serde_json::from_str(line)?;
    if value["command"] == "spawn" {
        serde_json::from_value(value).map(Command::Spawn)
    } else {
        serde_json::from_value(value).map(Command::Routed)
    }
}

fn route(command: Routed, stations: &Stations) {
    let Some(intents) = stations.get(&command.station) else {
        eprintln!("unknown station {}", command.station);
        return;
    };
    if intents.send(command.intent).is_err() {
        eprintln!("station {} stopped", command.station);
    }
}

fn spawn(csms: &str, command: &Spawn, events: &UnboundedSender<Event>, stations: &mut Stations) {
    for n in 1..=command.count {
        let id = format!("{}{n}", command.prefix);
        if stations.contains_key(&id) {
            eprintln!("station {id} already exists");
            continue;
        }
        let url = format!("{csms}/{id}");
        let station = Station {
            id: id.clone(),
            vendor: "ocppd".into(),
            model: "sim".into(),
            serial_number: id.clone(),
            firmware_version: env!("CARGO_PKG_VERSION").into(),
            voltage_v: 230.0,
            max_current_a: 32.0,
            phases: 3,
            jitter: command.jitter,
            connectors: vec![Connector::default()],
        };
        match ChargePoint::new(station) {
            Ok(mut charge_point) => {
                let refused = command.configuration.iter().find(|(key, value)| {
                    charge_point.configure(key, value) != ConfigurationStatus::Accepted
                });
                if let Some((key, _)) = refused {
                    eprintln!("cannot spawn station {id}: {key} refused");
                    continue;
                }
                let (intents, receiver) = unbounded_channel();
                stations.insert(id, intents);
                tokio::spawn(shell::run(charge_point, url, events.clone(), receiver));
            }
            Err(err) => eprintln!("cannot spawn station {id}: {err}"),
        }
    }
}

async fn write_events(mut receiver: UnboundedReceiver<Event>) {
    let mut stdout = tokio::io::stdout();
    while let Some(event) = receiver.recv().await {
        let mut line = match serde_json::to_string(&event) {
            Ok(line) => line,
            Err(err) => {
                eprintln!("event not serializable: {err}");
                continue;
            }
        };
        line.push('\n');
        let written = match stdout.write_all(line.as_bytes()).await {
            Ok(()) => stdout.flush().await,
            Err(err) => Err(err),
        };
        if let Err(err) = written {
            eprintln!("stdout: {err}");
            std::process::exit(1);
        }
    }
}
