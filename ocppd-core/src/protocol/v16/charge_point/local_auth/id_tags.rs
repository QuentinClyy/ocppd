use crate::protocol::v16::messages::datatypes::id_tag::{IdTagInfo, IdToken};

#[derive(Debug, Clone, Default)]
pub struct IdTags(Vec<(IdToken, IdTagInfo)>);

impl IdTags {
    pub fn find(&self, id_tag: &IdToken) -> Option<&IdTagInfo> {
        self.0
            .iter()
            .find(|(known, _)| known.matches(id_tag))
            .map(|(_, info)| info)
    }

    pub fn set(&mut self, id_tag: &IdToken, info: Option<&IdTagInfo>) {
        self.0.retain(|(known, _)| !known.matches(id_tag));
        if let Some(info) = info {
            self.0.push((id_tag.clone(), info.clone()));
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
#[path = "id_tags.test.rs"]
mod tests;
