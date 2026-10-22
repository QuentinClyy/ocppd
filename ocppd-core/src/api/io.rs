use crate::api::event::Event;
use crate::api::intent::Intent;
use crate::wire::frame::Frame;

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Connected,
    ConnectFailed,
    Disconnected,
    Tick,
    Frame(String),
    Intent(Intent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Send(Frame),
    Emit(Event),
    Close,
}
