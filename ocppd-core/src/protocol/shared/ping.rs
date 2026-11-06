pub const WEB_SOCKET_PING_INTERVAL: &str = "WebSocketPingInterval";
pub const DEFAULT_PING_SECONDS: u64 = 30;

pub fn next(
    scheduled: Option<u64>,
    now: u64,
    seconds: u64,
    connected: bool,
) -> (Option<u64>, bool) {
    if !connected || seconds == 0 {
        return (None, false);
    }
    let interval = seconds * 1000;
    match scheduled {
        Some(at) if at <= now => (Some(now + interval), true),
        Some(at) => (Some(at), false),
        None => (Some(now + interval), false),
    }
}

#[cfg(test)]
#[path = "ping.test.rs"]
mod tests;
