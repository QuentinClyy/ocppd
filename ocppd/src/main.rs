mod basic_auth;
mod cli;
mod identity;
mod model;
mod protocol;
mod shell;
mod tls;
mod tui;

use identity::Identities;
use model::Model;
use ocppd_core::api::event::Event;
use ocppd_core::api::intent::Intent;
use protocol::{Protocol, Version};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::process::ExitCode;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Deserialize)]
struct Spawn {
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let csms = match cli::parse(&args, std::env::var("OCPPD_CSMS").ok()) {
        Ok(cli::Cli::Run { csms, json }) => {
            if !json
                && tui::interactive()
                && let Some(code) = open_tui(&csms)
            {
                return code;
            }
            csms
        }
        Ok(cli::Cli::Help) => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(cli::Cli::Version) => {
            println!("ocppd {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(err) => {
            eprintln!("ocppd: {err}\n{}\nTry 'ocppd --help' for more.", cli::USAGE);
            return ExitCode::from(2);
        }
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

fn open_tui(csms: &str) -> Option<ExitCode> {
    let ocppd = std::env::current_exe().ok();
    let beside = ocppd.as_deref().and_then(|exe| exe.parent());
    let path = std::env::var_os("PATH");
    let (Some(ocppd), Some(tui)) = (ocppd.as_deref(), tui::find(beside, path.as_deref())) else {
        eprintln!("ocppd: ocppd-tui not found, reading JSON lines (--json hides this note)");
        return None;
    };
    let err = tui::open(&tui, csms, ocppd);
    eprintln!("ocppd: cannot open {}: {err}", tui.display());
    Some(ExitCode::FAILURE)
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
    let identities = match command.identities.list() {
        Ok(identities) => identities,
        Err(err) => {
            eprintln!("cannot spawn: {err}");
            return;
        }
    };
    for identity in identities {
        let id = identity.id;
        if stations.contains_key(&id) {
            eprintln!("station {id} already exists");
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
                eprintln!("cannot spawn station {id}: {err}");
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
                    eprintln!(
                        "cannot spawn station {id}: {key} refused{}",
                        refusal_hint(key)
                    );
                    continue;
                }
                let password = identity.password.as_deref();
                if password.is_some_and(|password| !protocol.set_basic_auth_password(password)) {
                    eprintln!("cannot spawn station {id}: password refused");
                    continue;
                }
                if !protocol.set_csms_url(base) {
                    eprintln!("cannot spawn station {id}: csms url too long");
                    continue;
                }
                let (intents, receiver) = unbounded_channel();
                stations.insert(id, intents);
                tokio::spawn(shell::run(protocol, url, events.clone(), receiver));
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
