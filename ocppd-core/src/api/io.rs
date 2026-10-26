use crate::api::event::Event;
use crate::api::intent::Intent;
use crate::wire::frame::Frame;

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Connected,
    ConnectFailed(ConnectFailure, String),
    Disconnected,
    Tick,
    Frame(String),
    Intent(Intent),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectFailure {
    CsmsCertificate,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Send(Frame),
    Emit(Event),
    Ping,
    Close,
}
