use super::id_tags::IdTags;
use crate::protocol::v16::messages::datatypes::id_tag::{IdTagInfo, IdToken};
use crate::protocol::v16::messages::local_auth::{SendLocalListRequest, UpdateStatus, UpdateType};

#[derive(Default)]
pub struct LocalList {
    version: i32,
    tags: IdTags,
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
        self.tags.find(id_tag)
    }

    pub fn len(&self) -> usize {
        self.tags.len()
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
        let mut tags = match request.update_type {
            UpdateType::Full => IdTags::default(),
            UpdateType::Differential => self.tags.clone(),
        };
        for change in changes {
            tags.set(&change.id_tag, change.id_tag_info.as_ref());
        }
        if tags.len() > limits.total {
            return UpdateStatus::Failed;
        }
        self.tags = tags;
        self.version = request.list_version;
        UpdateStatus::Accepted
    }
}

#[cfg(test)]
#[path = "list.test.rs"]
mod tests;
