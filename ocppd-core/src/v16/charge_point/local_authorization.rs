use super::ChargePoint;
use super::configuration::{
    AUTHORIZATION_CACHE_ENABLED, LOCAL_AUTH_LIST_ENABLED, LOCAL_AUTHORIZE_OFFLINE,
    LOCAL_PRE_AUTHORIZE,
};
use crate::v16::messages::datatypes::id_tag::{AuthorizationStatus, IdTagInfo, IdToken};

impl ChargePoint {
    pub(super) fn locally_authorized(&self, id_tag: &IdToken, now: u64) -> Option<IdTagInfo> {
        let when = match self.is_connected() {
            true => LOCAL_PRE_AUTHORIZE,
            false => LOCAL_AUTHORIZE_OFFLINE,
        };
        if !self.configuration.flag(when) {
            return None;
        }
        let listed = self
            .local_list
            .find(id_tag)
            .filter(|_| self.configuration.flag(LOCAL_AUTH_LIST_ENABLED));
        let cached = self
            .cache
            .find(id_tag)
            .filter(|_| self.configuration.flag(AUTHORIZATION_CACHE_ENABLED));
        let info = listed.or(cached)?;
        let valid = info.status == AuthorizationStatus::Accepted
            && info.expiry_date.is_none_or(|expiry| expiry.unix_ms > now);
        valid.then(|| info.clone())
    }

    pub(super) fn remember(&mut self, id_tag: &IdToken, info: &IdTagInfo) {
        if self.configuration.flag(AUTHORIZATION_CACHE_ENABLED) {
            self.cache.set(id_tag, Some(info));
        }
    }
}

#[cfg(test)]
#[path = "local_authorization.test.rs"]
mod tests;
