use crate::v201::messages::datatypes::id_token::{IdToken, IdTokenInfo};

#[derive(Default, Clone)]
pub struct Tokens {
    entries: Vec<Entry>,
}

#[derive(Clone)]
pub struct Entry {
    pub id_token: IdToken,
    pub info: IdTokenInfo,
    pub updated_at: u64,
}

impl Tokens {
    pub fn find(&self, id_token: &IdToken) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|entry| same(&entry.id_token, id_token))
    }

    pub fn set(&mut self, id_token: &IdToken, info: IdTokenInfo, now: u64) {
        self.remove(id_token);
        self.entries.push(Entry {
            id_token: id_token.clone(),
            info,
            updated_at: now,
        });
    }

    pub fn remove(&mut self, id_token: &IdToken) {
        self.entries
            .retain(|entry| !same(&entry.id_token, id_token));
    }

    pub fn retain(&mut self, kept: impl Fn(&Entry) -> bool) {
        self.entries.retain(kept);
    }

    pub fn evict_one(&mut self, first: impl Fn(&Entry) -> bool) {
        let victim = self.entries.iter().position(first).unwrap_or(0);
        if victim < self.entries.len() {
            self.entries.remove(victim);
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

fn same(left: &IdToken, right: &IdToken) -> bool {
    left.kind == right.kind && left.id_token.matches(&right.id_token)
}
