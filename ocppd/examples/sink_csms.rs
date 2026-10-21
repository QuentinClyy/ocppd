use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderValue;

const NOW: &str = "2026-01-01T00:00:00Z";

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(address), Some(interval)) = (args.first(), args.get(1)) else {
        eprintln!("usage: sink_csms <address> <heartbeat interval s>");
        std::process::exit(2);
    };
    let interval: u32 = interval.parse().expect("interval in seconds");
    let listener = TcpListener::bind(address).await.expect("bind");
    let calls = Arc::new(AtomicU64::new(0));
    let open = Arc::new(AtomicU64::new(0));
    tokio::spawn(report(calls.clone(), open.clone()));
    loop {
        let Ok((tcp, _)) = listener.accept().await else {
            continue;
        };
        tokio::spawn(serve(tcp, interval, calls.clone(), open.clone()));
    }
}

async fn report(calls: Arc<AtomicU64>, open: Arc<AtomicU64>) {
    let mut last = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let total = calls.load(Ordering::Relaxed);
        let connections = open.load(Ordering::Relaxed);
        eprintln!("connections {connections} calls/s {}", (total - last) / 10);
        last = total;
    }
}

async fn serve(tcp: TcpStream, interval: u32, calls: Arc<AtomicU64>, open: Arc<AtomicU64>) {
    let Ok(mut socket) = tokio_tungstenite::accept_hdr_async(tcp, ocpp16).await else {
        return;
    };
    open.fetch_add(1, Ordering::Relaxed);
    let mut transaction_id = 0;
    while let Some(Ok(message)) = socket.next().await {
        let Ok(text) = message.to_text() else {
            continue;
        };
        let Ok(Value::Array(frame)) = serde_json::from_str(text) else {
            continue;
        };
        let [kind, id, action, _] = frame.as_slice() else {
            continue;
        };
        if kind != 2 {
            continue;
        }
        calls.fetch_add(1, Ordering::Relaxed);
        transaction_id += 1;
        let payload = response(
            action.as_str().unwrap_or_default(),
            interval,
            transaction_id,
        );
        let reply = json!([3, id, payload]).to_string();
        if socket.send(Message::text(reply)).await.is_err() {
            break;
        }
    }
    open.fetch_sub(1, Ordering::Relaxed);
}

fn response(action: &str, interval: u32, transaction_id: i32) -> Value {
    let accepted = json!({"status": "Accepted"});
    match action {
        "BootNotification" => {
            json!({"status": "Accepted", "currentTime": NOW, "interval": interval})
        }
        "Heartbeat" => json!({"currentTime": NOW}),
        "Authorize" => json!({"idTagInfo": accepted}),
        "StartTransaction" => json!({"idTagInfo": accepted, "transactionId": transaction_id}),
        _ => json!({}),
    }
}

#[allow(clippy::result_large_err)]
fn ocpp16(_: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    let protocol = HeaderValue::from_static("ocpp1.6");
    response
        .headers_mut()
        .insert("Sec-WebSocket-Protocol", protocol);
    Ok(response)
}
