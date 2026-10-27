use crate::clients::hub::{Client, Control};
use crate::shell::dispatch::{Ended, Request, Shell};
use std::process::ExitCode;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub async fn run(shell: Shell, csms: &str) -> ExitCode {
    let (stdout, lines) = unbounded_channel();
    if shell.control.send(Control::Attach(stdout.clone())).is_err() {
        return ExitCode::FAILURE;
    }
    let writer = tokio::spawn(write(lines));
    tokio::spawn(read(shell.requests.clone(), stdout));
    if let Ended::Shutdown = shell.run(csms).await {
        return ExitCode::SUCCESS;
    }
    match writer.await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("event writer failed: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn read(requests: UnboundedSender<Request>, client: Client) {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                let request = Request {
                    line,
                    client: client.clone(),
                };
                if requests.send(request).is_err() {
                    return;
                }
            }
            Ok(None) => return,
            Err(err) => {
                eprintln!("stdin: {err}");
                return;
            }
        }
    }
}

async fn write(mut lines: UnboundedReceiver<String>) {
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
