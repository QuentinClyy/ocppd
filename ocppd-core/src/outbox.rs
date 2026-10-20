use crate::frame::Frame;
use serde_json::Value;
use std::collections::VecDeque;

const CALL_TIMEOUT_MS: u64 = 30_000;

pub struct Outbox {
    queue: VecDeque<(String, Value)>,
    pending: Option<Pending>,
    last_id: u64,
}

struct Pending {
    id: String,
    action: String,
    deadline: u64,
}

impl Outbox {
    pub fn new() -> Self {
        Outbox {
            queue: VecDeque::new(),
            pending: None,
            last_id: 0,
        }
    }

    pub fn push(&mut self, action: &str, payload: Value) {
        self.queue.push_back((action.to_owned(), payload));
    }

    pub fn next_frame(&mut self, now: u64) -> Option<Frame> {
        if self.pending.is_some() {
            return None;
        }
        let (action, payload) = self.queue.pop_front()?;
        self.last_id += 1;
        let id = self.last_id.to_string();
        self.pending = Some(Pending {
            id: id.clone(),
            action: action.clone(),
            deadline: now + CALL_TIMEOUT_MS,
        });
        Some(Frame::Call {
            id,
            action,
            payload,
        })
    }

    pub fn resolve(&mut self, id: &str) -> Option<String> {
        if self.pending.as_ref()?.id != id {
            return None;
        }
        self.pending.take().map(|pending| pending.action)
    }

    pub fn expire(&mut self, now: u64) -> Option<String> {
        if self.pending.as_ref()?.deadline > now {
            return None;
        }
        self.pending.take().map(|pending| pending.action)
    }

    pub fn deadline(&self) -> Option<u64> {
        self.pending.as_ref().map(|pending| pending.deadline)
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.pending = None;
    }
}
