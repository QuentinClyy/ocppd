mod harness;

use futures_util::StreamExt;
use harness::{SPAWN, bounded, first_error, launch, plugged, rejections, start};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

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
    let reconnected = bounded(run.daemon.listener.accept()).await;
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

#[tokio::test]
async fn given_a_station_spawned_without_connecting_when_read_then_offline_and_no_connection_until_connect()
 {
    let mut daemon = launch(r#"{"command":"spawn","prefix":"CP","count":1,"connect":false}"#).await;
    daemon.event("offline").await;
    let early = timeout(Duration::from_millis(300), daemon.listener.accept()).await;

    daemon
        .command(r#"{"command":"connect","station":"CP1"}"#)
        .await;

    assert!(early.is_err(), "dialed while held offline");
    daemon.event("online").await;
    let mut run = daemon.accept(300).await;
    run.event("connected").await;
}

#[tokio::test]
async fn given_a_connected_station_when_disconnect_then_connect_then_it_dials_again_at_once() {
    let mut run = start(SPAWN, 300).await;
    run.command(r#"{"command":"disconnect","station":"CP1"}"#)
        .await;
    run.event("offline").await;
    run.event("disconnected").await;

    run.command(r#"{"command":"connect","station":"CP1"}"#)
        .await;

    let reconnected = timeout(Duration::from_secs(1), run.daemon.listener.accept()).await;
    assert!(reconnected.is_ok(), "no dial within a second of connect");
}

#[tokio::test]
async fn given_an_offline_station_when_its_csms_is_changed_then_connect_reaches_the_new_one() {
    let mut daemon = launch(r#"{"command":"spawn","prefix":"CP","count":1,"connect":false}"#).await;
    daemon.event("offline").await;
    let other = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://127.0.0.1:{}", other.local_addr().unwrap().port());
    daemon
        .command(&format!(
            r#"{{"command":"set_csms","station":"CP1","url":"{url}"}}"#
        ))
        .await;
    daemon.event("csms_changed").await;

    daemon
        .command(r#"{"command":"connect","station":"CP1"}"#)
        .await;

    let reached = timeout(Duration::from_secs(1), other.accept()).await;
    assert!(reached.is_ok(), "the new csms was not dialed");
}

#[tokio::test]
async fn given_a_connected_station_when_its_csms_is_changed_then_it_is_refused() {
    let mut run = start(SPAWN, 300).await;

    run.command(r#"{"command":"set_csms","station":"CP1","url":"ws://127.0.0.1:1"}"#)
        .await;

    let rejected = run.event("intent_rejected").await;
    assert_eq!(rejected["reason"], "disconnect first");
}
