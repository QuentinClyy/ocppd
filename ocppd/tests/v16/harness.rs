use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderValue;

pub const SPAWN: &str = r#"{"command":"spawn","prefix":"CP","count":1}"#;

pub type Socket = WebSocketStream<TcpStream>;

pub struct Run {
    pub _child: Child,
    pub stdin: ChildStdin,
    pub stdout: Lines<BufReader<ChildStdout>>,
    pub socket: Socket,
    pub authorization: Option<String>,
    pub listener: TcpListener,
}

pub async fn bounded<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(10), future)
        .await
        .expect("scenario timed out")
}

pub async fn plugged() -> Run {
    let mut run = start(SPAWN, 3600).await;
    run.command(r#"{"command":"plug_in","station":"CP1","connector":1}"#)
        .await;
    let id = run.call("StatusNotification").await;
    run.reply(&id, json!({})).await;
    run
}

pub async fn first_error(scheme: &str, spawn: &str) -> (String, bool) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let csms = format!("{scheme}://127.0.0.1:{port}");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args(["--csms", &csms])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let line = format!("{}\n", spawn.replace('\n', ""));
    stdin.write_all(line.as_bytes()).await.unwrap();
    let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
    let error = bounded(stderr.next_line()).await.unwrap().unwrap();
    let accepted = timeout(Duration::from_millis(200), listener.accept()).await;
    (error, accepted.is_ok())
}

pub async fn start(spawn: &str, heartbeat_interval: u32) -> Run {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let csms = format!("ws://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let mut child = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args(["--csms", &csms])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap()).lines();
    stdin
        .write_all(format!("{spawn}\n").as_bytes())
        .await
        .unwrap();
    let (tcp, _) = bounded(listener.accept()).await.unwrap();
    let mut authorization = None;
    #[allow(clippy::result_large_err)]
    let record = |request: &Request, response: Response| {
        let header = request.headers().get("Authorization");
        authorization = header
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        check_handshake(request, response)
    };
    let socket = bounded(tokio_tungstenite::accept_hdr_async(tcp, record))
        .await
        .unwrap();
    let mut run = Run {
        _child: child,
        stdin,
        stdout,
        socket,
        authorization,
        listener,
    };

    let id = run.call("BootNotification").await;
    let accepted = json!({
        "status": "Accepted",
        "currentTime": "2026-10-20T00:00:00Z",
        "interval": heartbeat_interval
    });
    run.reply(&id, accepted).await;
    let id = run.call("SecurityEventNotification").await;
    run.reply(&id, json!({})).await;
    for _ in 0..2 {
        let id = run.call("StatusNotification").await;
        run.reply(&id, json!({})).await;
    }
    run
}

impl Run {
    pub async fn command(&mut self, line: &str) {
        self.stdin.write_all(line.as_bytes()).await.unwrap();
        self.stdin.write_all(b"\n").await.unwrap();
    }

    pub async fn next_call(&mut self) -> Value {
        let message = bounded(self.socket.next()).await.unwrap().unwrap();
        let frame: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
        assert_eq!(frame[0], 2);
        frame
    }

    pub async fn call(&mut self, action: &str) -> String {
        let frame = self.next_call().await;
        assert_eq!(frame[2], action, "{frame}");
        frame[1].as_str().unwrap().into()
    }

    pub async fn reply(&mut self, id: &str, payload: Value) {
        let text = json!([3, id, payload]).to_string();
        self.socket.send(Message::text(text)).await.unwrap();
    }

    pub async fn event(&mut self, kind: &str) -> Value {
        loop {
            let line = bounded(self.stdout.next_line()).await.unwrap().unwrap();
            let event: Value = serde_json::from_str(&line).unwrap();
            if event["kind"] == kind {
                return event;
            }
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn check_handshake(
    request: &Request,
    mut response: Response,
) -> Result<Response, ErrorResponse> {
    assert_eq!(request.uri().path(), "/CP1");
    assert_eq!(request.headers()["Sec-WebSocket-Protocol"], "ocpp1.6");
    response.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static("ocpp1.6"),
    );
    Ok(response)
}

pub async fn rejections(lines: &[&str]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args(["--csms", "ws://127.0.0.1:1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for line in lines {
        stdin
            .write_all(format!("{line}\n").as_bytes())
            .await
            .unwrap();
    }
    drop(stdin);
    let output = bounded(child.wait_with_output()).await.unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|event| event["kind"] == "command_rejected")
        .collect()
}
