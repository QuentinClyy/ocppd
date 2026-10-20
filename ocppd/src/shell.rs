use futures_util::{SinkExt, StreamExt};
use ocppd_core::event::{Event, Input, Output};
use ocppd_core::frame;
use ocppd_core::station::Station;
use ocppd_core::v16::charge_point::ChargePoint;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const RECONNECT_DELAY: Duration = Duration::from_secs(5);

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn run(station: Station, url: String, events: UnboundedSender<Event>) {
    let id = station.id.clone();
    let mut charge_point = ChargePoint::new(station);
    loop {
        match connect(&url).await {
            Ok(socket) => {
                let reason = serve(&id, &mut charge_point, socket, &events).await;
                eprintln!("{id}: disconnected: {reason}");
                for output in charge_point.handle(now_ms(), Input::Disconnected) {
                    if let Output::Emit(event) = output {
                        emit(&events, event);
                    }
                }
            }
            Err(err) => eprintln!("{id}: connect to {url} failed: {err}"),
        }
        sleep(RECONNECT_DELAY).await;
    }
}

async fn connect(url: &str) -> Result<Socket, tungstenite::Error> {
    let mut request = url.into_client_request()?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static("ocpp1.6"),
    );
    let (socket, _) = connect_async(request).await?;
    Ok(socket)
}

async fn serve(
    id: &str,
    charge_point: &mut ChargePoint,
    mut socket: Socket,
    events: &UnboundedSender<Event>,
) -> String {
    let mut input = Input::Connected;
    loop {
        for output in charge_point.handle(now_ms(), input) {
            match output {
                Output::Send(frame) => {
                    let text = frame::serialize(&frame);
                    if let Err(err) = socket.send(Message::text(text)).await {
                        return err.to_string();
                    }
                }
                Output::Emit(event) => emit(events, event),
            }
        }
        input = match next_input(id, &mut socket, charge_point.next_deadline()).await {
            Ok(input) => input,
            Err(reason) => return reason,
        };
    }
}

async fn next_input(id: &str, socket: &mut Socket, deadline: Option<u64>) -> Result<Input, String> {
    let wait = Duration::from_millis(deadline.unwrap_or(0).saturating_sub(now_ms()));
    let timer = sleep(wait);
    tokio::pin!(timer);
    loop {
        tokio::select! {
            message = socket.next() => match message {
                Some(Ok(Message::Text(text))) => return Ok(Input::Frame(text.to_string())),
                Some(Ok(Message::Binary(_))) => eprintln!("{id}: ignored binary message"),
                Some(Ok(Message::Close(_))) | None => return Err("closed by CSMS".into()),
                Some(Ok(_)) => {}
                Some(Err(err)) => return Err(err.to_string()),
            },
            _ = &mut timer, if deadline.is_some() => return Ok(Input::Tick),
        }
    }
}

fn emit(events: &UnboundedSender<Event>, event: Event) {
    if events.send(event).is_err() {
        eprintln!("event dropped: writer stopped");
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}
