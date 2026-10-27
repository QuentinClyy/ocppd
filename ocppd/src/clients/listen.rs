use crate::clients::hub::Control;
use crate::shell::dispatch::Request;
use serde_json::Value;
use std::fmt;
use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener, UnixStream};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

const BACKLOG: usize = 10_000;
const HELLO_WITHIN: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Unix(PathBuf),
    Tcp(String),
}

impl Address {
    pub fn parse(text: &str) -> Result<Address, String> {
        match text.split_once(':') {
            Some(("unix", path)) if !path.is_empty() => Ok(Address::Unix(path.into())),
            Some(("tcp", host)) if !host.is_empty() => Ok(Address::Tcp(host.into())),
            _ => Err(format!(
                "listen address '{text}' must be unix:<path> or tcp:<host>:<port>"
            )),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Address::Unix(path) => write!(f, "unix:{}", path.display()),
            Address::Tcp(host) => write!(f, "tcp:{host}"),
        }
    }
}

pub enum Listener {
    Unix(UnixListener),
    Tcp(TcpListener),
}

#[derive(Clone)]
pub struct Session {
    pub requests: UnboundedSender<Request>,
    pub control: UnboundedSender<Control>,
    pub greeting: String,
}

pub async fn bind(address: &Address) -> io::Result<Listener> {
    match address {
        Address::Unix(path) => {
            if UnixStream::connect(path).await.is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "another daemon is listening there",
                ));
            }
            match std::fs::remove_file(path) {
                Err(err) if err.kind() != io::ErrorKind::NotFound => return Err(err),
                _ => {}
            }
            let listener = UnixListener::bind(path)?;
            std::fs::set_permissions(path, Permissions::from_mode(0o600))?;
            Ok(Listener::Unix(listener))
        }
        Address::Tcp(host) => Ok(Listener::Tcp(TcpListener::bind(host).await?)),
    }
}

pub async fn serve(listener: Listener, session: Session, token: Option<String>) {
    loop {
        let accepted = match &listener {
            Listener::Unix(unix) => unix
                .accept()
                .await
                .map(|(stream, _)| tokio::spawn(client(stream, session.clone(), None))),
            Listener::Tcp(tcp) => tcp
                .accept()
                .await
                .map(|(stream, _)| tokio::spawn(client(stream, session.clone(), token.clone()))),
        };
        if let Err(err) = accepted {
            eprintln!("accept: {err}");
        }
    }
}

async fn client<S>(stream: S, session: Session, token: Option<String>)
where
    S: AsyncRead + AsyncWrite + Send + 'static,
{
    let (read, mut write) = tokio::io::split(stream);
    let mut lines = BufReader::new(read).lines();
    if let Some(token) = token {
        let hello = tokio::time::timeout(HELLO_WITHIN, lines.next_line()).await;
        let Ok(Ok(Some(line))) = hello else {
            return;
        };
        if !greets(&line, &token) {
            return;
        }
    }
    let (client, mut outbox) = unbounded_channel();
    if client.send(session.greeting.clone()).is_err()
        || session
            .control
            .send(Control::Attach(client.clone()))
            .is_err()
    {
        return;
    }
    loop {
        tokio::select! {
            line = lines.next_line() => match line {
                Ok(Some(line)) => {
                    let request = Request { line, client: client.clone() };
                    if session.requests.send(request).is_err() {
                        return;
                    }
                }
                _ => return,
            },
            Some(mut line) = outbox.recv() => {
                if outbox.len() > BACKLOG {
                    eprintln!("client too slow, disconnected");
                    return;
                }
                line.push('\n');
                if write.write_all(line.as_bytes()).await.is_err() {
                    return;
                }
            }
        }
    }
}

fn greets(line: &str, token: &str) -> bool {
    serde_json::from_str::<Value>(line)
        .is_ok_and(|hello| hello["command"] == "hello" && hello["token"] == token)
}

#[cfg(test)]
#[path = "listen.test.rs"]
mod tests;
