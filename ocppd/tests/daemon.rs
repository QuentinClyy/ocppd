use futures_util::StreamExt;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines};
use tokio::net::{TcpListener, TcpStream, UnixStream};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderValue;

struct Daemon {
    child: Child,
    socket: PathBuf,
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket);
    }
}

struct Client<S> {
    lines: Lines<BufReader<tokio::io::ReadHalf<S>>>,
    write: tokio::io::WriteHalf<S>,
}

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(10), future)
        .await
        .expect("scenario timed out")
}

fn socket_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ocppd-daemon-{}-{name}.sock", std::process::id()))
}

async fn daemon(name: &str, csms: &str) -> Daemon {
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

async fn connect(daemon: &Daemon) -> Client<UnixStream> {
    client(UnixStream::connect(&daemon.socket).await.unwrap())
}

fn client<S: AsyncRead + AsyncWrite>(stream: S) -> Client<S> {
    let (read, write) = tokio::io::split(stream);
    Client {
        lines: BufReader::new(read).lines(),
        write,
    }
}

impl<S: AsyncRead + AsyncWrite> Client<S> {
    async fn send(&mut self, line: &str) {
        self.write
            .write_all(format!("{line}\n").as_bytes())
            .await
            .unwrap();
    }

    async fn next(&mut self) -> Option<Value> {
        let line = bounded(self.lines.next_line()).await.ok()??;
        Some(serde_json::from_str(&line).unwrap())
    }

    async fn until(&mut self, kind: &str) -> Value {
        loop {
            let event = self.next().await.expect("connection closed");
            if event["kind"] == kind {
                return event;
            }
        }
    }

    async fn quiet(&mut self) -> bool {
        timeout(Duration::from_millis(300), self.lines.next_line())
            .await
            .is_err()
    }
}

async fn fake_csms() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://127.0.0.1:{}", listener.local_addr().unwrap().port());
    (listener, url)
}

#[allow(clippy::result_large_err)]
fn ocpp16(_: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    let protocol = HeaderValue::from_static("ocpp1.6");
    response
        .headers_mut()
        .insert("Sec-WebSocket-Protocol", protocol);
    Ok(response)
}

#[tokio::test]
async fn given_two_clients_when_they_connect_then_each_gets_the_daemon_line_first() {
    let daemon = daemon("greeting", "ws://127.0.0.1:1").await;

    let mut first = connect(&daemon).await;
    let mut second = connect(&daemon).await;

    for client in [&mut first, &mut second] {
        let greeting = client.next().await.unwrap();
        assert_eq!(greeting["kind"], "daemon");
        assert_eq!(greeting["csms"], "ws://127.0.0.1:1");
        assert_eq!(greeting["version"], env!("CARGO_PKG_VERSION"));
    }
}

#[tokio::test]
async fn given_a_command_that_cannot_run_when_one_client_sends_it_then_only_that_client_is_answered()
 {
    let daemon = daemon("reply", "ws://127.0.0.1:1").await;
    let mut asking = connect(&daemon).await;
    let mut other = connect(&daemon).await;
    asking.until("daemon").await;
    other.until("daemon").await;

    asking
        .send(r#"{"command":"stop","station":"NOPE","connector":1}"#)
        .await;

    assert_eq!(
        asking.until("command_rejected").await["reason"],
        "unknown station"
    );
    assert!(other.quiet().await);
}

#[tokio::test]
async fn given_a_station_spawned_by_a_client_that_left_when_another_connects_then_it_still_runs_and_shows_in_the_snapshot()
 {
    let (csms, url) = fake_csms().await;
    let daemon = daemon("outlive", &url).await;
    let mut spawning = connect(&daemon).await;
    spawning
        .send(r#"{"command":"spawn","prefix":"CP","count":1}"#)
        .await;
    let (tcp, _) = bounded(csms.accept()).await.unwrap();
    let mut station = bounded(tokio_tungstenite::accept_hdr_async(tcp, ocpp16))
        .await
        .unwrap();
    spawning.until("connected").await;

    drop(spawning);
    let boot = bounded(station.next()).await.unwrap().unwrap();
    let mut later = connect(&daemon).await;

    assert!(boot.to_text().unwrap().contains("BootNotification"));
    later.until("daemon").await;
    let connected = later.until("connected").await;
    assert_eq!(connected["station"], "CP1");
}

#[tokio::test]
async fn given_a_running_daemon_when_shutdown_is_sent_then_it_exits_and_removes_its_socket() {
    let mut daemon = daemon("shutdown", "ws://127.0.0.1:1").await;
    let mut client = connect(&daemon).await;

    client.send(r#"{"command":"shutdown"}"#).await;

    let status = bounded(daemon.child.wait()).await.unwrap();
    assert!(status.success());
    assert!(!daemon.socket.exists());
}

#[tokio::test]
async fn given_a_running_daemon_when_terminated_then_it_exits_and_removes_its_socket() {
    let mut daemon = daemon("terminate", "ws://127.0.0.1:1").await;
    let pid = daemon.child.id().unwrap().to_string();

    Command::new("kill")
        .args(["-TERM", &pid])
        .status()
        .await
        .unwrap();

    let status = bounded(daemon.child.wait()).await.unwrap();
    assert!(status.success());
    assert!(!daemon.socket.exists());
}

#[tokio::test]
async fn given_a_daemon_on_a_socket_when_a_second_one_starts_there_then_it_is_refused() {
    let running = daemon("twice", "ws://127.0.0.1:1").await;

    let second = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args([
            "--csms",
            "ws://127.0.0.1:1",
            "--listen",
            &format!("unix:{}", running.socket.display()),
        ])
        .output()
        .await
        .unwrap();

    assert_eq!(second.status.code(), Some(1));
    let stderr = String::from_utf8(second.stderr).unwrap();
    assert!(
        stderr.contains("another daemon is listening there"),
        "{stderr}"
    );
}

#[tokio::test]
async fn given_tcp_without_a_token_when_started_then_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args(["--csms", "ws://127.0.0.1:1", "--listen", "tcp:127.0.0.1:0"])
        .env_remove("OCPPD_TOKEN")
        .output()
        .await
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("OCPPD_TOKEN"), "{stderr}");
}

async fn tcp_daemon(token: &str) -> (Child, String) {
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

#[tokio::test]
async fn given_a_tcp_client_with_a_wrong_token_when_it_says_hello_then_it_is_disconnected() {
    let (_daemon, address) = tcp_daemon("s3cret").await;
    let mut client = client(TcpStream::connect(&address).await.unwrap());

    client.send(r#"{"command":"hello","token":"guess"}"#).await;

    assert_eq!(client.next().await, None);
}

#[tokio::test]
async fn given_a_tcp_client_with_the_token_when_it_says_hello_then_it_gets_the_daemon_line() {
    let (_daemon, address) = tcp_daemon("s3cret").await;
    let mut client = client(TcpStream::connect(&address).await.unwrap());

    client.send(r#"{"command":"hello","token":"s3cret"}"#).await;

    assert_eq!(client.next().await.unwrap()["kind"], "daemon");
}
