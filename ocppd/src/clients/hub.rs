use crate::clients::snapshot::Snapshot;
use ocppd_core::api::event::Event;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type Client = UnboundedSender<String>;

pub enum Control {
    Attach(Client),
    Forget(String),
}

pub async fn run(mut events: UnboundedReceiver<Event>, mut control: UnboundedReceiver<Control>) {
    let mut snapshot = Snapshot::default();
    let mut clients: Vec<Client> = Vec::new();
    loop {
        tokio::select! {
            Some(event) = events.recv() => {
                let line = match serde_json::to_string(&event) {
                    Ok(line) => line,
                    Err(err) => {
                        eprintln!("event not serializable: {err}");
                        continue;
                    }
                };
                snapshot.apply(&line);
                clients.retain(|client| client.send(line.clone()).is_ok());
            }
            Some(message) = control.recv() => match message {
                Control::Attach(client) => {
                    if snapshot.lines().into_iter().all(|line| client.send(line).is_ok()) {
                        clients.push(client);
                    }
                }
                Control::Forget(station) => snapshot.forget(&station),
            },
            else => return,
        }
    }
}

#[cfg(test)]
#[path = "hub.test.rs"]
mod tests;
