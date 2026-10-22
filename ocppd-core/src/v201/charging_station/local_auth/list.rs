use super::tokens::Tokens;
use crate::v201::messages::local_auth::{SendLocalListRequest, SendLocalListStatus, UpdateType};

pub const CAPACITY: usize = 1000;
pub const ITEMS_PER_MESSAGE: usize = 100;

#[derive(Default)]
pub struct LocalList {
    pub version: i32,
    pub tokens: Tokens,
}

impl LocalList {
    pub fn update(&mut self, request: &SendLocalListRequest, now: u64) -> SendLocalListStatus {
        let changes = request
            .local_authorization_list
            .as_deref()
            .unwrap_or_default();
        let differential = request.update_type == UpdateType::Differential;
        if differential && request.version_number <= self.version {
            return SendLocalListStatus::VersionMismatch;
        }
        let duplicate = changes.iter().enumerate().any(|(index, change)| {
            changes[..index].iter().any(|earlier| {
                earlier.id_token.kind == change.id_token.kind
                    && earlier.id_token.id_token.matches(&change.id_token.id_token)
            })
        });
        if request.version_number <= 0 || changes.len() > ITEMS_PER_MESSAGE || duplicate {
            return SendLocalListStatus::Failed;
        }
        let mut tokens = match request.update_type {
            UpdateType::Full => Tokens::default(),
            UpdateType::Differential => self.tokens.clone(),
        };
        for change in changes {
            match &change.id_token_info {
                Some(info) => tokens.set(&change.id_token, info.clone(), now),
                None => tokens.remove(&change.id_token),
            }
        }
        if tokens.len() > CAPACITY {
            return SendLocalListStatus::Failed;
        }
        self.tokens = tokens;
        self.version = request.version_number;
        SendLocalListStatus::Accepted
    }
}

#[cfg(test)]
#[path = "list.test.rs"]
mod tests;
