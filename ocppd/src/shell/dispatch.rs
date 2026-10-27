use crate::clients::hub::{Client, Control};
use crate::shell::commands::{self, Command, Stations};
use crate::shell::rejection::Rejection;
use ocppd_core::api::event::Event;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub const SHUTDOWN: &str = r#"{"command":"shutdown"}"#;

pub struct Request {
    pub line: String,
    pub client: Client,
}

pub enum Ended {
    Closed,
    Shutdown,
}

pub struct Shell {
    pub events: UnboundedSender<Event>,
    pub control: UnboundedSender<Control>,
    pub requests: UnboundedSender<Request>,
    inbox: UnboundedReceiver<Request>,
}

impl Shell {
    pub fn start() -> Shell {
        let (events, event_inbox) = unbounded_channel();
        let (control, control_inbox) = unbounded_channel();
        tokio::spawn(crate::clients::hub::run(event_inbox, control_inbox));
        let (requests, inbox) = unbounded_channel();
        Shell {
            events,
            control,
            requests,
            inbox,
        }
    }

    pub async fn run(self, csms: &str) -> Ended {
        let Shell {
            events,
            control,
            requests,
            inbox,
        } = self;
        drop(requests);
        dispatch(csms, events, control, inbox).await
    }
}

async fn dispatch(
    csms: &str,
    events: UnboundedSender<Event>,
    control: UnboundedSender<Control>,
    mut requests: UnboundedReceiver<Request>,
) -> Ended {
    let mut stations = Stations::new();
    while let Some(Request { line, client }) = requests.recv().await {
        if line.trim().is_empty() {
            continue;
        }
        let rejections = match commands::parse(&line) {
            Ok(Command::Shutdown) => {
                commands::stop(stations).await;
                return Ended::Shutdown;
            }
            Ok(Command::Spawn(command)) => commands::spawn(csms, &command, &events, &mut stations),
            Ok(Command::Remove(command)) => match commands::remove(&command, &mut stations) {
                Ok(()) => {
                    let _ = control.send(Control::Forget(command.station));
                    Vec::new()
                }
                Err(rejection) => vec![rejection],
            },
            Ok(Command::Routed(command)) => commands::route(command, &stations)
                .err()
                .into_iter()
                .collect(),
            Err(rejection) => vec![rejection],
        };
        for rejection in rejections {
            reply(&client, &rejection);
        }
    }
    Ended::Closed
}

fn reply(client: &Client, rejection: &Rejection) {
    match serde_json::to_string(rejection) {
        Ok(line) => {
            let _ = client.send(line);
        }
        Err(err) => eprintln!("rejection not serializable: {err}"),
    }
}
