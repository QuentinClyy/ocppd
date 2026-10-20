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

type Socket = WebSocketStream<TcpStream>;

struct Run {
    _child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    socket: Socket,
    authorization: Option<String>,
}

const SPAWN: &str = r#"{"command":"spawn","prefix":"CP","count":1}"#;
const AUTHORIZE: &str = r#"{"command":"authorize","station":"CP1","connector":1,"id_tag":"TAG1"}"#;

#[tokio::test]
async fn given_csms_listening_when_station_spawned_then_connects_without_basic_auth() {
    let spawn = SPAWN;

    let run = start(spawn, 1).await;

    assert_eq!(run.authorization, None);
}

#[tokio::test]
async fn given_booted_station_when_stdout_read_then_connected_and_boot_status_reported() {
    let mut run = start(SPAWN, 1).await;

    let events = [run.event("connected").await, run.event("boot_status").await];

    assert_eq!(events[0]["station"], "CP1");
    assert_eq!(events[1]["status"], "Accepted");
}

#[tokio::test]
async fn given_authorized_tag_when_start_transaction_accepted_then_transaction_started_on_stdout() {
    let mut run = plugged().await;
    run.command(AUTHORIZE).await;
    let id = run.call("Authorize").await;
    run.reply(&id, json!({"idTagInfo": {"status": "Accepted"}}))
        .await;
    let id = run.call("StartTransaction").await;

    let started = json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 7});
    run.reply(&id, started).await;

    let event = run.event("transaction_started").await;
    assert_eq!(event["transaction_id"], 7);
    assert_eq!(event["connector"], 1);
}

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(10), future)
        .await
        .expect("scenario timed out")
}

async fn plugged() -> Run {
    let mut run = start(SPAWN, 3600).await;
    run.command(r#"{"command":"plug_in","station":"CP1","connector":1}"#)
        .await;
    let id = run.call("StatusNotification").await;
    run.reply(&id, json!({})).await;
    run
}

async fn start(spawn: &str, heartbeat_interval: u32) -> Run {
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
    async fn command(&mut self, line: &str) {
        self.stdin.write_all(line.as_bytes()).await.unwrap();
        self.stdin.write_all(b"\n").await.unwrap();
    }

    async fn next_call(&mut self) -> Value {
        let message = bounded(self.socket.next()).await.unwrap().unwrap();
        let frame: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
        assert_eq!(frame[0], 2);
        frame
    }

    async fn call(&mut self, action: &str) -> String {
        let frame = self.next_call().await;
        assert_eq!(frame[2], action, "{frame}");
        frame[1].as_str().unwrap().into()
    }

    async fn reply(&mut self, id: &str, payload: Value) {
        let text = json!([3, id, payload]).to_string();
        self.socket.send(Message::text(text)).await.unwrap();
    }

    async fn event(&mut self, kind: &str) -> Value {
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
fn check_handshake(request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    assert_eq!(request.uri().path(), "/CP1");
    assert_eq!(request.headers()["Sec-WebSocket-Protocol"], "ocpp1.6");
    response.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static("ocpp1.6"),
    );
    Ok(response)
}
