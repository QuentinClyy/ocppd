mod attach;
mod basic_auth;
mod cli;
mod commands;
mod daemon;
mod dispatch;
mod hub;
mod identity;
mod link;
mod listen;
mod model;
mod protocol;
mod rejection;
mod snapshot;
mod stdio;
mod tls;
mod tui;

use dispatch::Shell;
use listen::Address;
use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (csms, json, addresses) = match cli::parse(&args, std::env::var("OCPPD_CSMS").ok()) {
        Ok(cli::Cli::Run { csms, json, listen }) => (csms, json, listen),
        Ok(cli::Cli::Help) => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(cli::Cli::Version) => {
            println!("ocppd {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(err) => return usage_error(&err),
    };
    if !addresses.is_empty() {
        let token = std::env::var("OCPPD_TOKEN")
            .ok()
            .filter(|token| !token.is_empty());
        let tcp = addresses
            .iter()
            .any(|address| matches!(address, Address::Tcp(_)));
        if tcp && token.is_none() {
            return usage_error("listening on tcp needs OCPPD_TOKEN");
        }
        return daemon::run(Shell::start(), &csms, &addresses, token).await;
    }
    if !json
        && tui::interactive()
        && let Some(code) = open_tui(&csms)
    {
        return code;
    }
    stdio::run(Shell::start(), &csms).await
}

fn usage_error(err: &str) -> ExitCode {
    eprintln!("ocppd: {err}\n{}\nTry 'ocppd --help' for more.", cli::USAGE);
    ExitCode::from(2)
}

fn open_tui(csms: &str) -> Option<ExitCode> {
    let ocppd = std::env::current_exe().ok();
    let beside = ocppd.as_deref().and_then(|exe| exe.parent());
    let path = std::env::var_os("PATH");
    let (Some(ocppd), Some(tui)) = (ocppd.as_deref(), tui::find(beside, path.as_deref())) else {
        eprintln!("ocppd: ocppd-tui not found, reading JSON lines (--json hides this note)");
        return None;
    };
    let env = |key: &str| std::env::var(key).ok();
    let socket = attach::default_socket(env);
    if let Err(err) = attach::ensure(&socket, csms, ocppd, &attach::log_file(env)) {
        eprintln!("ocppd: {err}");
        return Some(ExitCode::FAILURE);
    }
    let err = tui::open(&tui, &format!("unix:{}", socket.display()));
    eprintln!("ocppd: cannot open {}: {err}", tui.display());
    Some(ExitCode::FAILURE)
}
