use futures_util::StreamExt;
use serde_json::Value;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::{Child, Command};
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

type Socket = WebSocketStream<TcpStream>;

#[tokio::test]
async fn given_2_0_1_station_spawned_when_connecting_then_offers_its_subprotocol_without_basic_auth()
 {
    let configuration = "{}";

    let handshake = connect_2_0_1(configuration).await;

    assert_eq!(handshake.protocol.as_deref(), Some("ocpp2.0.1"));
    assert_eq!(handshake.authorization, None);
}

#[tokio::test]
async fn given_connected_2_0_1_station_when_its_first_frame_arrives_then_power_up_boot_notification()
 {
    let mut handshake = connect_2_0_1("{}").await;

    let message = bounded(handshake.socket.next()).await.unwrap().unwrap();

    let frame: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
    assert_eq!(frame[2], "BootNotification");
    assert_eq!(frame[3]["reason"], "PowerUp");
    assert_eq!(frame[3]["chargingStation"]["serialNumber"], "CS1");
}

#[tokio::test]
async fn given_security_profile_1_and_password_when_connecting_then_basic_auth_per_a00_fr_202_to_205()
 {
    let configuration = r#"{"SecurityCtrlr.SecurityProfile":"1","SecurityCtrlr.BasicAuthPassword":"correct-horse-battery"}"#;

    let handshake = connect_2_0_1(configuration).await;

    let expected = format!("Basic {}", "Q1MxOmNvcnJlY3QtaG9yc2UtYmF0dGVyeQ==");
    assert_eq!(handshake.authorization, Some(expected));
}

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    timeout(Duration::from_secs(10), future)
        .await
        .expect("scenario timed out")
}

struct Handshake {
    _child: Child,
    socket: Socket,
    protocol: Option<String>,
    authorization: Option<String>,
}

async fn connect_2_0_1(configuration: &str) -> Handshake {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let csms = format!("ws://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let mut child = Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args(["--csms", "ws://127.0.0.1:1"])
        .stdin(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let spawn = format!(
        r#"{{"command":"spawn","prefix":"CS","count":1,"ocpp":"2.0.1","csms":"{csms}","configuration":{configuration}}}"#
    );
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(format!("{spawn}\n").as_bytes())
        .await
        .unwrap();
    let (tcp, _) = bounded(listener.accept()).await.unwrap();
    let mut protocol = None;
    let mut authorization = None;
    #[allow(clippy::result_large_err)]
    let record = |request: &Request, mut response: Response| {
        let offered = request.headers()["Sec-WebSocket-Protocol"].clone();
        protocol = offered.to_str().ok().map(str::to_owned);
        authorization = request
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        response
            .headers_mut()
            .insert("Sec-WebSocket-Protocol", offered);
        Ok(response)
    };
    let socket = bounded(tokio_tungstenite::accept_hdr_async(tcp, record))
        .await
        .unwrap();
    Handshake {
        _child: child,
        socket,
        protocol,
        authorization,
    }
}
