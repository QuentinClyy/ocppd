use super::tokens::Tokens;
use crate::v201::messages::datatypes::id_token::{AuthorizationStatus, IdToken, IdTokenInfo};

pub const CAPACITY: usize = 1000;

#[derive(Default)]
pub struct Cache {
    pub tokens: Tokens,
}

impl Cache {
    pub fn lookup(
        &mut self,
        id_token: &IdToken,
        now: u64,
        lifetime_ms: u64,
    ) -> Option<IdTokenInfo> {
        self.tokens.retain(|entry| {
            let expired = entry
                .info
                .cache_expiry_date_time
                .is_some_and(|at| at.unix_ms <= now);
            !expired && now < entry.updated_at + lifetime_ms
        });
        self.tokens.find(id_token).map(|entry| entry.info.clone())
    }

    pub fn remember(&mut self, id_token: &IdToken, info: IdTokenInfo, now: u64) {
        let expired = info
            .cache_expiry_date_time
            .is_some_and(|at| at.unix_ms <= now);
        if expired {
            return self.tokens.remove(id_token);
        }
        if self.tokens.find(id_token).is_none() && self.tokens.len() >= CAPACITY {
            self.tokens
                .evict_one(|entry| entry.info.status != AuthorizationStatus::Accepted);
        }
        self.tokens.set(id_token, info, now);
    }
}

#[cfg(test)]
#[path = "cache.test.rs"]
mod tests;
