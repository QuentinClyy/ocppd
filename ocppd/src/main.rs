mod shell;

use ocppd_core::event::Event;
use ocppd_core::station::{Connector, Station};
use ocppd_core::v16::charge_point::ChargePoint;
use serde::Deserialize;
use std::process::ExitCode;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum Command {
    Spawn { prefix: String, count: u32 },
}

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
        match serde_json::from_str(&line) {
            Ok(Command::Spawn { prefix, count }) => spawn(csms, &prefix, count, &events),
            Err(err) => eprintln!("bad command: {err}"),
        }
    }
}

fn spawn(csms: &str, prefix: &str, count: u32, events: &UnboundedSender<Event>) {
    for n in 1..=count {
        let id = format!("{prefix}{n}");
        let url = format!("{csms}/{id}");
        let station = Station {
            id: id.clone(),
            vendor: "ocppd".into(),
            model: "sim".into(),
            serial_number: id,
            firmware_version: env!("CARGO_PKG_VERSION").into(),
            voltage_v: 230.0,
            max_current_a: 32.0,
            phases: 3,
            connectors: vec![Connector::default()],
        };
        match ChargePoint::new(station) {
            Ok(charge_point) => {
                tokio::spawn(shell::run(charge_point, url, events.clone()));
            }
            Err(err) => eprintln!("cannot spawn station: {err}"),
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
