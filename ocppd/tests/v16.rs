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
    listener: TcpListener,
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
async fn given_station_booted_with_a_1_second_interval_when_a_second_passes_then_heartbeat_sent() {
    let mut run = start(SPAWN, 1).await;

    let frame = run.next_call().await;

    assert_eq!(frame[2], "Heartbeat");
}

#[tokio::test]
async fn given_security_profile_1_and_authorization_key_when_connecting_then_basic_auth_with_the_key()
 {
    let spawn = r#"{"command":"spawn","prefix":"CP","count":1,
        "configuration":{"SecurityProfile":"1","AuthorizationKey":"30313233343536373839616263646566"}}"#;

    let run = start(&spawn.replace('\n', ""), 300).await;

    let expected = "Basic Q1AxOjAxMjM0NTY3ODlhYmNkZWY=";
    assert_eq!(run.authorization.as_deref(), Some(expected));
}

#[tokio::test]
async fn given_a_listed_station_with_its_own_password_when_connecting_then_basic_auth_with_it() {
    let spawn = r#"{"command":"spawn","configuration":{"SecurityProfile":"1"},
        "stations":[{"id":"CP1","password":"0123456789abcdef"}]}"#;

    let run = start(&spawn.replace('\n', ""), 300).await;

    let expected = "Basic Q1AxOjAxMjM0NTY3ODlhYmNkZWY=";
    assert_eq!(run.authorization.as_deref(), Some(expected));
}

#[tokio::test]
async fn given_a_password_and_no_security_profile_when_connecting_then_basic_auth_is_sent() {
    let spawn = r#"{"command":"spawn","stations":[{"id":"CP1","password":"0123456789abcdef"}]}"#;

    let run = start(spawn, 300).await;

    let expected = "Basic Q1AxOjAxMjM0NTY3ODlhYmNkZWY=";
    assert_eq!(run.authorization.as_deref(), Some(expected));
}

#[tokio::test]
async fn given_security_profile_2_when_spawned_with_a_ws_url_then_refused_without_connecting() {
    let spawn = r#"{"command":"spawn","prefix":"CP","count":1,
        "configuration":{"AuthorizationKey":"30313233343536373839616263646566","SecurityProfile":"2"}}"#;

    let (error, connected) = first_error("ws", spawn).await;

    assert!(error.contains("requires a wss:// url"), "{error}");
    assert!(!connected, "no connection attempted");
}

#[tokio::test]
async fn given_security_profile_3_without_certificate_when_spawned_then_refused_without_connecting()
{
    let spawn = r#"{"command":"spawn","prefix":"CP","count":1,
        "configuration":{"SecurityProfile":"3"}}"#;

    let (error, connected) = first_error("wss", spawn).await;

    assert!(
        error.contains("requires a charge point certificate"),
        "{error}"
    );
    assert!(!connected, "no connection attempted");
}

#[tokio::test]
async fn given_a_connected_station_when_removed_then_its_socket_closes_and_its_id_is_free_again() {
    let mut run = start(SPAWN, 300).await;

    run.command(r#"{"command":"remove","station":"CP1"}"#).await;

    let closed = bounded(run.socket.next()).await;
    assert!(!matches!(closed, Some(Ok(Message::Text(_)))), "{closed:?}");
    run.command(SPAWN).await;
    let reconnected = bounded(run.listener.accept()).await;
    assert!(reconnected.is_ok());
}

#[tokio::test]
async fn given_plugged_station_when_tag_authorized_on_stdin_then_authorize_sent() {
    let mut run = plugged().await;

    run.command(AUTHORIZE).await;

    let frame = run.next_call().await;
    assert_eq!(frame[2], "Authorize");
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

async fn first_error(scheme: &str, spawn: &str) -> (String, bool) {
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

#[tokio::test]
async fn given_commands_that_cannot_run_when_sent_then_each_is_rejected_on_stdout() {
    let lines = [
        r#"{"command":"stop","station":"NOPE","connector":1}"#,
        r#"{"command":"remove","station":"NOPE"}"#,
        r#"{"command":"spawn","prefix":"CP","count":1,"phases":4}"#,
        "not json",
    ];

    let rejected = rejections(&lines).await;

    let reasons: Vec<_> = rejected
        .iter()
        .map(|event| {
            (
                event["command"].clone(),
                event["station"].clone(),
                event["reason"].clone(),
            )
        })
        .collect();
    assert_eq!(
        reasons,
        [
            (json!("stop"), json!("NOPE"), json!("unknown station")),
            (json!("remove"), json!("NOPE"), json!("unknown station")),
            (json!("spawn"), json!("CP1"), json!("phases must be 1 to 3")),
            (
                Value::Null,
                Value::Null,
                json!("expected ident at line 1 column 2")
            ),
        ]
    );
}

async fn rejections(lines: &[&str]) -> Vec<Value> {
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
