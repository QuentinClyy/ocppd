use crate::wire::frame::Frame;
use serde_json::Value;
use std::collections::VecDeque;

const CALL_TIMEOUT_MS: u64 = 30_000;
const SENT_BEFORE_REGISTRATION: &str = "BootNotification";

pub struct Outbox {
    transaction_messages: &'static [&'static str],
    queue: VecDeque<Queued>,
    pending: Option<Pending>,
    last_id: u64,
    instructing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub action: String,
    pub connector_id: u32,
}

pub struct Sent {
    pub call: Call,
    pub attempts: u32,
    payload: Value,
}

impl Sent {
    pub fn payload(&self) -> &Value {
        &self.payload
    }
}

struct Queued {
    sent: Sent,
    not_before: u64,
    instructed: bool,
}

struct Pending {
    id: String,
    sent: Sent,
    deadline: u64,
}

impl Outbox {
    pub fn new(transaction_messages: &'static [&'static str]) -> Self {
        Outbox {
            transaction_messages,
            queue: VecDeque::new(),
            pending: None,
            last_id: 0,
            instructing: false,
        }
    }

    pub fn push(&mut self, action: &str, connector_id: u32, payload: Value) {
        let call = Call {
            action: action.to_owned(),
            connector_id,
        };
        let sent = Sent {
            call,
            attempts: 0,
            payload,
        };
        self.queue.push_back(Queued {
            sent,
            not_before: 0,
            instructed: self.instructing,
        });
    }

    pub fn set_instructing(&mut self, instructing: bool) {
        self.instructing = instructing;
    }

    pub fn next_frame(&mut self, now: u64, registered: bool) -> Option<Frame> {
        if self.pending.is_some() {
            return None;
        }
        let mut transactions_waiting = false;
        let index = self.queue.iter().position(|queued| {
            let transactional = self.is_transaction_message(&queued.sent.call.action);
            let waiting = queued.not_before > now || (transactional && transactions_waiting);
            transactions_waiting |= transactional && waiting;
            let allowed = registered
                || queued.instructed
                || queued.sent.call.action == SENT_BEFORE_REGISTRATION;
            allowed && !waiting
        })?;
        let mut sent = self.queue.remove(index)?.sent;
        sent.attempts += 1;
        self.last_id += 1;
        let id = self.last_id.to_string();
        let frame = Frame::Call {
            id: id.clone(),
            action: sent.call.action.clone(),
            payload: sent.payload.clone(),
        };
        self.pending = Some(Pending {
            id,
            sent,
            deadline: now + CALL_TIMEOUT_MS,
        });
        Some(frame)
    }

    pub fn resolve(&mut self, id: &str) -> Option<Sent> {
        if self.pending.as_ref()?.id != id {
            return None;
        }
        self.pending.take().map(|pending| pending.sent)
    }

    pub fn expire(&mut self, now: u64) -> Option<Sent> {
        if self.pending.as_ref()?.deadline > now {
            return None;
        }
        self.pending.take().map(|pending| pending.sent)
    }

    pub fn retry(&mut self, sent: Sent, not_before: u64) {
        self.queue.push_front(Queued {
            sent,
            not_before,
            instructed: false,
        });
    }

    pub fn holds(&self, held: impl Fn(&str, &Value) -> bool) -> bool {
        let pending = self.pending.iter().map(|pending| &pending.sent);
        let queued = self.queue.iter().map(|queued| &queued.sent);
        pending
            .chain(queued)
            .any(|sent| held(&sent.call.action, &sent.payload))
    }

    pub fn queued_payloads(
        &mut self,
        connector_id: u32,
    ) -> impl Iterator<Item = (&str, &mut Value)> {
        self.queue
            .iter_mut()
            .filter(move |queued| queued.sent.call.connector_id == connector_id)
            .map(|queued| (queued.sent.call.action.as_str(), &mut queued.sent.payload))
    }

    pub fn is_drained(&self, now: u64) -> bool {
        self.pending.is_none() && self.queue.iter().all(|queued| queued.not_before > now)
    }

    pub fn deadline(&self, registered: bool) -> Option<u64> {
        if let Some(pending) = &self.pending {
            return Some(pending.deadline);
        }
        let waiting = self.queue.iter().map(|queued| queued.not_before);
        waiting.filter(|&at| at > 0).min().filter(|_| registered)
    }

    pub fn keep(&mut self, kept: impl Fn(&str, &Value) -> bool) {
        if let Some(Pending { mut sent, .. }) = self.pending.take() {
            sent.attempts = sent.attempts.saturating_sub(1);
            self.queue.push_front(Queued {
                sent,
                not_before: 0,
                instructed: false,
            });
        }
        self.queue
            .retain(|queued| kept(&queued.sent.call.action, &queued.sent.payload));
    }

    fn is_transaction_message(&self, action: &str) -> bool {
        self.transaction_messages.contains(&action)
    }
}
