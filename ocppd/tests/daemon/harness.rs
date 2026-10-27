use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines};
use tokio::net::{TcpListener, TcpStream, UnixStream};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderValue;

pub struct Daemon {
    pub child: Child,
    pub socket: PathBuf,
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket);
    }
}

pub struct Client<S> {
    pub lines: Lines<BufReader<tokio::io::ReadHalf<S>>>,
    pub write: tokio::io::WriteHalf<S>,
}

pub async fn bounded<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(10), future)
        .await
        .expect("scenario timed out")
}

pub fn socket_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ocppd-daemon-{}-{name}.sock", std::process::id()))
}

pub async fn daemon(name: &str, csms: &str) -> Daemon {
    let socket = socket_path(name);
    let child = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args([
            "--csms",
            csms,
            "--listen",
            &format!("unix:{}", socket.display()),
        ])
        .env_remove("OCPPD_TOKEN")
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    bounded(async {
        while UnixStream::connect(&socket).await.is_err() {
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    Daemon { child, socket }
}

pub async fn connect(daemon: &Daemon) -> Client<UnixStream> {
    client(UnixStream::connect(&daemon.socket).await.unwrap())
}

pub fn client<S: AsyncRead + AsyncWrite>(stream: S) -> Client<S> {
    let (read, write) = tokio::io::split(stream);
    Client {
        lines: BufReader::new(read).lines(),
        write,
    }
}

impl<S: AsyncRead + AsyncWrite> Client<S> {
    pub async fn send(&mut self, line: &str) {
        self.write
            .write_all(format!("{line}\n").as_bytes())
            .await
            .unwrap();
    }

    pub async fn next(&mut self) -> Option<Value> {
        let line = bounded(self.lines.next_line()).await.ok()??;
        Some(serde_json::from_str(&line).unwrap())
    }

    pub async fn until(&mut self, kind: &str) -> Value {
        loop {
            let event = self.next().await.expect("connection closed");
            if event["kind"] == kind {
                return event;
            }
        }
    }

    pub async fn quiet(&mut self) -> bool {
        timeout(Duration::from_millis(300), self.lines.next_line())
            .await
            .is_err()
    }
}

pub async fn fake_csms() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://127.0.0.1:{}", listener.local_addr().unwrap().port());
    (listener, url)
}

#[allow(clippy::result_large_err)]
pub fn ocpp16(_: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    let protocol = HeaderValue::from_static("ocpp1.6");
    response
        .headers_mut()
        .insert("Sec-WebSocket-Protocol", protocol);
    Ok(response)
}

pub async fn tcp_daemon(token: &str) -> (Child, String) {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let address = format!("127.0.0.1:{port}");
    let child = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args([
            "--csms",
            "ws://127.0.0.1:1",
            "--listen",
            &format!("tcp:{address}"),
        ])
        .env("OCPPD_TOKEN", token)
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    bounded(async {
        while TcpStream::connect(&address).await.is_err() {
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    (child, address)
}

pub async fn answer_until_closed(mut station: WebSocketStream<TcpStream>) -> bool {
    while let Some(Ok(message)) = station.next().await {
        let frame: Value = match message {
            Message::Text(text) => serde_json::from_str(&text).unwrap(),
            Message::Close(_) => return true,
            _ => continue,
        };
        let payload = match frame[2].as_str() {
            Some("BootNotification") => {
                json!({"status": "Accepted", "currentTime": "2026-10-20T00:00:00Z", "interval": 300})
            }
            _ => json!({}),
        };
        let reply = json!([3, frame[1], payload]).to_string();
        if station.send(Message::text(reply)).await.is_err() {
            return false;
        }
    }
    false
}
