mod basic_auth;
mod cli;
mod commands;
mod hub;
mod identity;
mod model;
mod protocol;
mod rejection;
mod shell;
mod snapshot;
mod tls;
mod tui;

use commands::{Command, Stations};
use hub::{Client, Control};
use ocppd_core::api::event::Event;
use rejection::Rejection;
use std::process::ExitCode;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

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
    let (events, event_inbox) = unbounded_channel();
    let (control, control_inbox) = unbounded_channel();
    tokio::spawn(hub::run(event_inbox, control_inbox));
    let (stdout, lines) = unbounded_channel();
    if control.send(Control::Attach(stdout.clone())).is_err() {
        return ExitCode::FAILURE;
    }
    let writer = tokio::spawn(write_lines(lines));
    read_commands(&csms, events, control, stdout).await;
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

async fn read_commands(
    csms: &str,
    events: UnboundedSender<Event>,
    control: UnboundedSender<Control>,
    client: Client,
) {
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
        let rejections = match commands::parse(&line) {
            Ok(Command::Spawn(command)) => commands::spawn(csms, &command, &events, &mut stations),
            Ok(Command::Remove(command)) => match commands::remove(&command, &mut stations) {
                Ok(()) => {
                    let _ = control.send(Control::Forget(command.station));
                    Vec::new()
                }
                Err(rejection) => vec![rejection],
            },
            Ok(Command::Routed(command)) => commands::route(command, &stations)
                .err()
                .into_iter()
                .collect(),
            Err(rejection) => vec![rejection],
        };
        for rejection in rejections {
            if !reply(&client, &rejection) {
                return;
            }
        }
    }
}

fn reply(client: &Client, rejection: &Rejection) -> bool {
    match serde_json::to_string(rejection) {
        Ok(line) => client.send(line).is_ok(),
        Err(err) => {
            eprintln!("rejection not serializable: {err}");
            true
        }
    }
}

async fn write_lines(mut lines: UnboundedReceiver<String>) {
    let mut stdout = tokio::io::stdout();
    while let Some(mut line) = lines.recv().await {
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
