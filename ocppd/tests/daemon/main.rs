mod harness;

use futures_util::StreamExt;
use harness::{
    answer_until_closed, bounded, client, connect, daemon, fake_csms, ocpp16, tcp_daemon,
};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::timeout;

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
async fn given_a_spawned_station_when_one_client_asks_to_describe_it_then_only_that_client_gets_its_model_and_configuration()
 {
    let daemon = daemon("describe", "ws://127.0.0.1:1").await;
    let mut asking = connect(&daemon).await;
    let mut other = connect(&daemon).await;
    asking.until("daemon").await;
    other.until("daemon").await;
    asking
        .send(r#"{"command":"spawn","stations":[{"id":"CP1"}],"vendor":"ACME","configuration":{"HeartbeatInterval":"60"}}"#)
        .await;

    asking
        .send(r#"{"command":"describe","station":"CP1"}"#)
        .await;

    let described = asking.until("station_described").await;
    assert_eq!(described["vendor"], "ACME");
    assert_eq!(described["ocpp"], "1.6");
    assert_eq!(described["csms"], "ws://127.0.0.1:1/CP1");
    let heartbeat = described["configuration"]
        .as_array()
        .unwrap()
        .iter()
        .find(|setting| setting["key"] == "HeartbeatInterval")
        .unwrap();
    assert_eq!(heartbeat["value"], "60");
    let mut kinds = Vec::new();
    while let Ok(Some(line)) = timeout(Duration::from_millis(300), other.next()).await {
        kinds.push(line["kind"].clone());
    }
    assert!(!kinds.contains(&"station_described".into()));
}

#[tokio::test]
async fn given_no_such_station_when_described_then_the_asking_client_is_told_it_is_unknown() {
    let daemon = daemon("describe-unknown", "ws://127.0.0.1:1").await;
    let mut asking = connect(&daemon).await;
    asking.until("daemon").await;

    asking
        .send(r#"{"command":"describe","station":"NOPE"}"#)
        .await;

    let rejected = asking.until("command_rejected").await;
    assert_eq!(
        (rejected["command"].as_str(), rejected["reason"].as_str()),
        (Some("describe"), Some("unknown station"))
    );
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

#[tokio::test]
async fn given_a_tcp_daemon_when_clients_say_hello_then_only_the_one_with_the_token_gets_the_daemon_line()
 {
    let (_daemon, address) = tcp_daemon("s3cret").await;
    let mut guessing = client(TcpStream::connect(&address).await.unwrap());
    let mut knowing = client(TcpStream::connect(&address).await.unwrap());

    guessing
        .send(r#"{"command":"hello","token":"guess"}"#)
        .await;
    knowing
        .send(r#"{"command":"hello","token":"s3cret"}"#)
        .await;

    assert_eq!(guessing.next().await, None);
    assert_eq!(knowing.next().await.unwrap()["kind"], "daemon");
}

#[tokio::test]
async fn given_a_booted_station_when_shutdown_is_sent_then_it_closes_its_websocket_before_the_daemon_exits()
 {
    let (csms, url) = fake_csms().await;
    let mut daemon = daemon("graceful", &url).await;
    let mut client = connect(&daemon).await;
    client
        .send(r#"{"command":"spawn","prefix":"CP","count":1}"#)
        .await;
    let (tcp, _) = bounded(csms.accept()).await.unwrap();
    let station = bounded(tokio_tungstenite::accept_hdr_async(tcp, ocpp16))
        .await
        .unwrap();
    let closed = tokio::spawn(answer_until_closed(station));
    client.until("boot_status").await;

    client.send(r#"{"command":"shutdown"}"#).await;

    assert!(bounded(closed).await.unwrap(), "no close frame");
    assert!(bounded(daemon.child.wait()).await.unwrap().success());
}
