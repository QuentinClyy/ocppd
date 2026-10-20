use crate::v16::messages::datatypes::id_tag::{IdTagInfo, IdToken};
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

#[cfg(test)]
#[path = "local_list.test.rs"]
mod tests;
