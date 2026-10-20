use super::ChargePoint;
use super::configuration::{LOCAL_AUTH_LIST_ENABLED, LOCAL_AUTHORIZE_OFFLINE, LOCAL_PRE_AUTHORIZE};
use crate::v16::messages::datatypes::id_tag::{AuthorizationStatus, IdTagInfo, IdToken};
use crate::v16::messages::local_auth::{SendLocalListRequest, UpdateStatus, UpdateType};

#[derive(Default)]
pub struct LocalList {
    version: i32,
    entries: Vec<(IdToken, IdTagInfo)>,
}

pub struct Limits {
    pub per_update: usize,
    pub total: usize,
}

impl LocalList {
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn find(&self, id_tag: &IdToken) -> Option<&IdTagInfo> {
        self.entries
            .iter()
            .find(|(listed, _)| listed.matches(id_tag))
            .map(|(_, info)| info)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn update(&mut self, request: &SendLocalListRequest, limits: Limits) -> UpdateStatus {
        let changes = request
            .local_authorization_list
            .as_deref()
            .unwrap_or_default();
        let differential = request.update_type == UpdateType::Differential;
        if differential && request.list_version <= self.version {
            return UpdateStatus::VersionMismatch;
        }
        if request.list_version < 1 || changes.len() > limits.per_update {
            return UpdateStatus::Failed;
        }
        let mut entries = match request.update_type {
            UpdateType::Full => Vec::new(),
            UpdateType::Differential => self.entries.clone(),
        };
        for change in changes {
            entries.retain(|(id_tag, _)| !id_tag.matches(&change.id_tag));
            if let Some(info) = &change.id_tag_info {
                entries.push((change.id_tag.clone(), info.clone()));
            }
        }
        if entries.len() > limits.total {
            return UpdateStatus::Failed;
        }
        self.entries = entries;
        self.version = request.list_version;
        UpdateStatus::Accepted
    }
}

impl ChargePoint {
    pub(super) fn locally_authorized(&self, id_tag: &IdToken, now: u64) -> Option<IdTagInfo> {
        let when = match self.is_connected() {
            true => LOCAL_PRE_AUTHORIZE,
            false => LOCAL_AUTHORIZE_OFFLINE,
        };
        let enabled =
            self.configuration.flag(LOCAL_AUTH_LIST_ENABLED) && self.configuration.flag(when);
        let info = self.local_list.find(id_tag).filter(|_| enabled)?;
        let valid = info.status == AuthorizationStatus::Accepted
            && info.expiry_date.is_none_or(|expiry| expiry.unix_ms > now);
        valid.then(|| info.clone())
    }
}

#[cfg(test)]
#[path = "local_list.test.rs"]
mod tests;
