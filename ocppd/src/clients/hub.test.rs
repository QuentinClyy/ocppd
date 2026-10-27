use super::*;
use ocppd_core::api::event::{Kind, LinkEvent};
use ocppd_core::protocol::v16::event::EventKind;
use tokio::sync::mpsc::unbounded_channel;

fn connected(station: &str, at: u64) -> Event {
    Event {
        station: station.into(),
        at,
        kind: Kind::V16(EventKind::Link(LinkEvent::Connected)),
    }
}

fn drain(lines: &mut UnboundedReceiver<String>) -> Vec<String> {
    let mut drained = Vec::new();
    while let Ok(line) = lines.try_recv() {
        drained.push(line);
    }
    drained
}

struct Hub {
    events: UnboundedSender<Event>,
    control: UnboundedSender<Control>,
}

fn hub() -> Hub {
    let (events, event_inbox) = unbounded_channel();
    let (control, control_inbox) = unbounded_channel();
    tokio::spawn(run(event_inbox, control_inbox));
    Hub { events, control }
}

async fn settle() {
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}

fn attach(hub: &Hub) -> UnboundedReceiver<String> {
    let (client, lines) = unbounded_channel();
    hub.control.send(Control::Attach(client)).unwrap();
    lines
}

#[tokio::test]
async fn given_events_before_a_client_attaches_when_it_attaches_then_it_gets_them_as_a_snapshot() {
    let hub = hub();
    hub.events.send(connected("CP1", 1)).unwrap();
    hub.events.send(connected("CP2", 2)).unwrap();
    settle().await;

    let mut lines = attach(&hub);
    settle().await;

    let stations: Vec<_> = drain(&mut lines)
        .iter()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap()["station"].clone())
        .collect();
    assert_eq!(stations, ["CP1", "CP2"]);
}

#[tokio::test]
async fn given_two_attached_clients_when_an_event_arrives_then_both_get_it() {
    let hub = hub();
    let mut first = attach(&hub);
    let mut second = attach(&hub);
    settle().await;

    hub.events.send(connected("CP1", 1)).unwrap();
    settle().await;

    assert_eq!(drain(&mut first).len(), 1);
    assert_eq!(drain(&mut second).len(), 1);
}

#[tokio::test]
async fn given_a_client_that_left_when_an_event_arrives_then_the_others_still_get_it() {
    let hub = hub();
    drop(attach(&hub));
    let mut staying = attach(&hub);
    settle().await;

    hub.events.send(connected("CP1", 1)).unwrap();
    settle().await;

    assert_eq!(drain(&mut staying).len(), 1);
}

#[tokio::test]
async fn given_a_forgotten_station_when_a_client_attaches_then_its_snapshot_leaves_it_out() {
    let hub = hub();
    hub.events.send(connected("CP1", 1)).unwrap();
    hub.events.send(connected("CP2", 2)).unwrap();
    settle().await;
    hub.control.send(Control::Forget("CP1".into())).unwrap();

    let mut lines = attach(&hub);
    settle().await;

    let snapshot = drain(&mut lines);
    assert_eq!(snapshot.len(), 1);
    assert!(snapshot[0].contains("CP2"));
}
