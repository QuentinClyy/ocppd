use crate::dispatch::Shell;
use crate::listen::{self, Address, Session};
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::signal::unix::{SignalKind, signal};

pub async fn run(
    shell: Shell,
    csms: &str,
    addresses: &[Address],
    token: Option<String>,
) -> ExitCode {
    let session = Session {
        requests: shell.requests.clone(),
        control: shell.control.clone(),
        greeting: json!({"kind": "daemon", "csms": csms, "version": env!("CARGO_PKG_VERSION")})
            .to_string(),
    };
    let mut sockets: Vec<PathBuf> = Vec::new();
    for address in addresses {
        let listener = match listen::bind(address).await {
            Ok(listener) => listener,
            Err(err) => {
                eprintln!("ocppd: cannot listen on {address}: {err}");
                remove(&sockets);
                return ExitCode::FAILURE;
            }
        };
        let client_token = match address {
            Address::Unix(path) => {
                sockets.push(path.clone());
                None
            }
            Address::Tcp(_) => token.clone(),
        };
        tokio::spawn(listen::serve(listener, session.clone(), client_token));
    }
    drop(session);
    tokio::select! {
        _ = shell.run(csms) => {}
        _ = stopped() => {}
    }
    remove(&sockets);
    ExitCode::SUCCESS
}

async fn stopped() {
    match signal(SignalKind::terminate()) {
        Ok(mut terminate) => {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = terminate.recv() => {}
            }
        }
        Err(_) => {
            let _ = tokio::signal::ctrl_c().await;
        }
    }
}

fn remove(sockets: &[PathBuf]) {
    for path in sockets {
        if let Err(err) = std::fs::remove_file(path) {
            eprintln!("ocppd: cannot remove {}: {err}", path.display());
        }
    }
}
