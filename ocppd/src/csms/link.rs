use crate::csms::basic_auth;
use crate::csms::protocol::Protocol;
use crate::csms::tls;
use futures_util::{SinkExt, StreamExt};
use ocppd_core::api::event::Event;
use ocppd_core::api::intent::Intent;
use ocppd_core::api::io::{ConnectFailure, Input, Output};
use ocppd_core::station::jitter::fraction;
use ocppd_core::wire::frame;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::time::{Instant, sleep, sleep_until};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_tls_with_config};

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(300);
const READ_BUFFER_BYTES: usize = 4 * 1024;
const CERTIFICATE_REFUSED: &str = "certificate verify failed";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub enum Order {
    Intent(Intent),
    Describe(oneshot::Sender<String>),
}

type Orders = Option<UnboundedReceiver<Order>>;

struct Link {
    id: String,
    socket: Option<Socket>,
    failures: u32,
    reconnect_at: Instant,
}

enum Wake {
    Input(Input),
    Describe(oneshot::Sender<String>),
    Reconnect,
    Lost(String),
}

pub async fn run(
    mut protocol: Protocol,
    url: String,
    events: UnboundedSender<Event>,
    receiver: UnboundedReceiver<Order>,
    starts_online: bool,
) {
    let mut link = Link {
        id: protocol.station().id.clone(),
        socket: None,
        failures: 0,
        reconnect_at: Instant::now(),
    };
    if !starts_online {
        execute(
            &mut protocol,
            Input::Intent(Intent::Disconnect),
            &mut link.socket,
            &events,
        )
        .await;
    }
    let mut orders = Some(receiver);
    loop {
        let dials = protocol.wants_connection();
        let mut input =
            match next_wake(&mut link, protocol.next_deadline(), dials, &mut orders).await {
                Wake::Input(input) => input,
                Wake::Describe(reply) => {
                    let csms = target(&protocol, &url, &link.id);
                    let _ = reply.send(protocol.description(&csms, now_ms()));
                    continue;
                }
                Wake::Lost(reason) => link.lose(&reason, &protocol),
                Wake::Reconnect => {
                    let target = target(&protocol, &url, &link.id);
                    match connect(&target, &protocol).await {
                        Ok(socket) => {
                            link.socket = Some(socket);
                            link.failures = 0;
                            Input::Connected
                        }
                        Err(err) => {
                            eprintln!("{}: connect to {target} failed: {err}", link.id);
                            link.schedule_reconnect(&protocol);
                            Input::ConnectFailed(failure(&err), err)
                        }
                    }
                }
            };
        let held = !protocol.wants_connection();
        while let Some(reason) = execute(&mut protocol, input, &mut link.socket, &events).await {
            input = link.lose(&reason, &protocol);
        }
        if held && protocol.wants_connection() {
            link.failures = 0;
            link.reconnect_at = Instant::now();
        }
        if protocol.finished(now_ms()) {
            if let Some(mut open) = link.socket.take() {
                let _ = open.close(None).await;
            }
            return;
        }
    }
}

impl Link {
    fn lose(&mut self, reason: &str, protocol: &Protocol) -> Input {
        eprintln!("{}: disconnected: {reason}", self.id);
        self.socket = None;
        self.schedule_reconnect(protocol);
        Input::Disconnected
    }

    fn schedule_reconnect(&mut self, protocol: &Protocol) {
        self.failures += 1;
        let delay = match protocol.reconnect_delay_ms(self.failures) {
            Some(ms) => Duration::from_millis(ms),
            None => backoff(&self.id, self.failures, protocol.station().jitter),
        };
        self.reconnect_at = Instant::now() + delay;
    }
}

fn failure(err: &str) -> ConnectFailure {
    match err.contains(CERTIFICATE_REFUSED) {
        true => ConnectFailure::CsmsCertificate,
        false => ConnectFailure::Other,
    }
}

fn target(protocol: &Protocol, url: &str, id: &str) -> String {
    match protocol.csms_url() {
        Some(base) => format!("{}/{id}", base.trim_end_matches('/')),
        None => url.to_owned(),
    }
}

fn backoff(id: &str, failures: u32, jitter: bool) -> Duration {
    let doublings = failures.saturating_sub(1).min(8);
    let delay = (RECONNECT_DELAY * (1 << doublings)).min(MAX_RECONNECT_DELAY);
    match jitter {
        true => delay.mul_f64(0.5 + fraction(id, u64::from(failures))),
        false => delay,
    }
}

async fn connect(url: &str, protocol: &Protocol) -> Result<Socket, String> {
    let secure = url.starts_with("wss://");
    if protocol.requires_tls() && !secure {
        return Err("security profile requires a wss:// url".into());
    }
    if protocol.requires_client_certificate() && protocol.client_certificate().is_none() {
        return Err("security profile 3 requires a charge point certificate".into());
    }
    let id = &protocol.station().id;
    let request = request(
        url,
        id,
        protocol.subprotocol(),
        protocol.basic_auth_password(),
    );
    let request = request.map_err(|err| err.to_string())?;
    let tls = match secure {
        true => Some(tls::connector(protocol).map_err(|err| err.to_string())?),
        false => None,
    };
    let connected = connect_async_tls_with_config(request, Some(socket_config()), false, tls).await;
    let (socket, _) = connected.map_err(|err| err.to_string())?;
    Ok(socket)
}

fn socket_config() -> WebSocketConfig {
    WebSocketConfig::default()
        .read_buffer_size(READ_BUFFER_BYTES)
        .write_buffer_size(0)
}

fn request(
    url: &str,
    id: &str,
    subprotocol: &'static str,
    password: Option<Vec<u8>>,
) -> Result<Request, tungstenite::Error> {
    let mut request = url.into_client_request()?;
    let headers = request.headers_mut();
    headers.insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static(subprotocol),
    );
    if let Some(password) = password {
        let credentials = basic_auth::header(id, &password);
        let value = HeaderValue::from_str(&credentials).map_err(tungstenite::http::Error::from)?;
        headers.insert("Authorization", value);
    }
    Ok(request)
}

async fn execute(
    protocol: &mut Protocol,
    input: Input,
    socket: &mut Option<Socket>,
    events: &UnboundedSender<Event>,
) -> Option<String> {
    for output in protocol.handle(now_ms(), input) {
        match (output, socket.as_mut()) {
            (Output::Emit(event), _) => emit(events, event),
            (Output::Send(frame), Some(open)) => {
                let text = frame::serialize(&frame);
                if let Err(err) = open.send(Message::text(text)).await {
                    return Some(err.to_string());
                }
            }
            (Output::Send(frame), None) => eprintln!("frame dropped while offline: {frame:?}"),
            (Output::Ping, Some(open)) => {
                if let Err(err) = open.send(Message::Ping(Vec::new().into())).await {
                    return Some(err.to_string());
                }
            }
            (Output::Ping, None) => {}
            (Output::Close, Some(open)) => {
                return Some(match open.close(None).await {
                    Ok(()) => "closed by the station".into(),
                    Err(err) => format!("closed by the station: {err}"),
                });
            }
            (Output::Close, None) => {}
        }
    }
    None
}

async fn next_wake(
    link: &mut Link,
    deadline: Option<u64>,
    dials: bool,
    orders: &mut Orders,
) -> Wake {
    let wait = Duration::from_millis(deadline.unwrap_or(0).saturating_sub(now_ms()));
    let timer = sleep(wait);
    tokio::pin!(timer);
    let offline = link.socket.is_none() && dials;
    let reconnect = sleep_until(link.reconnect_at);
    tokio::pin!(reconnect);
    loop {
        tokio::select! {
            message = next_message(&mut link.socket) => match message {
                Some(Ok(Message::Text(text))) => return Wake::Input(Input::Frame(text.to_string())),
                Some(Ok(Message::Binary(_))) => eprintln!("{}: ignored binary message", link.id),
                Some(Ok(Message::Close(_))) | None => return Wake::Lost("closed by CSMS".into()),
                Some(Ok(_)) => {}
                Some(Err(err)) => return Wake::Lost(err.to_string()),
            },
            _ = &mut timer, if deadline.is_some() => return Wake::Input(Input::Tick),
            _ = &mut reconnect, if offline => return Wake::Reconnect,
            order = next_order(orders) => match order {
                Some(Order::Intent(intent)) => return Wake::Input(Input::Intent(intent)),
                Some(Order::Describe(reply)) => return Wake::Describe(reply),
                None => *orders = None,
            },
        }
    }
}

async fn next_message(socket: &mut Option<Socket>) -> Option<Result<Message, tungstenite::Error>> {
    match socket {
        Some(open) => open.next().await,
        None => std::future::pending().await,
    }
}

async fn next_order(orders: &mut Orders) -> Option<Order> {
    match orders {
        Some(receiver) => receiver.recv().await,
        None => std::future::pending().await,
    }
}

fn emit(events: &UnboundedSender<Event>, event: Event) {
    if events.send(event).is_err() {
        eprintln!("event dropped: writer stopped");
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

#[cfg(test)]
#[path = "link.test.rs"]
mod tests;
