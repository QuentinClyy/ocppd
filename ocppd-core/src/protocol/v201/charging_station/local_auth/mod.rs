pub mod cache;
pub mod list;
mod tokens;

use super::ChargingStation;
use super::device_model::table::AUTH_CTRLR;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::authorization::{ClearCacheResponse, ClearCacheStatus};
use crate::protocol::v201::messages::datatypes::id_token::{
    AuthorizationStatus, IdToken, IdTokenInfo,
};
use crate::protocol::v201::messages::local_auth::{
    GetLocalListVersionResponse, SendLocalListRequest, SendLocalListResponse, SendLocalListStatus,
};
use crate::wire::ci_string::CiString;

pub const AUTH_CACHE_CTRLR: &str = "AuthCacheCtrlr";
pub const LOCAL_AUTH_LIST_CTRLR: &str = "LocalAuthListCtrlr";
pub const ENABLED: &str = "Enabled";
pub const ENTRIES: &str = "Entries";

pub enum Decision {
    Local(Option<CiString<36>>),
    Ask,
    Refuse,
}

impl ChargingStation {
    pub(super) fn decide(&mut self, now: u64, id_token: &IdToken) -> Decision {
        let online = self.is_accepted();
        let accepted_here = match online {
            true => self.auth_flag("LocalPreAuthorize"),
            false => self.auth_flag("LocalAuthorizeOffline"),
        };
        let listed = self
            .flag(LOCAL_AUTH_LIST_CTRLR, ENABLED)
            .then(|| {
                self.local_list
                    .tokens
                    .find(id_token)
                    .map(|entry| entry.info.clone())
            })
            .flatten();
        let lifetime_ms = self
            .device_model
            .number(AUTH_CACHE_CTRLR, "LifeTime")
            .unwrap_or(0)
            * 1000;
        let cached = match listed.is_none() && self.flag(AUTH_CACHE_CTRLR, ENABLED) {
            true => self.cache.lookup(id_token, now, lifetime_ms),
            false => None,
        };
        match listed.or(cached) {
            Some(info) if info.status == AuthorizationStatus::Accepted && accepted_here => {
                Decision::Local(info.group_id_token.map(|group| group.id_token))
            }
            Some(_) | None if online => Decision::Ask,
            Some(_) => Decision::Refuse,
            None if self.auth_flag("OfflineTxForUnknownIdEnabled") => Decision::Local(None),
            None => Decision::Refuse,
        }
    }

    pub(super) fn remember(&mut self, id_token: &IdToken, info: IdTokenInfo, now: u64) {
        if self.flag(AUTH_CACHE_CTRLR, ENABLED) {
            self.cache.remember(id_token, info, now);
        }
    }

    pub(super) fn clear_cache(&mut self) -> ClearCacheResponse {
        let status = match self.flag(AUTH_CACHE_CTRLR, ENABLED) {
            true => {
                self.cache = cache::Cache::default();
                ClearCacheStatus::Accepted
            }
            false => ClearCacheStatus::Rejected,
        };
        ClearCacheResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn send_local_list(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: &SendLocalListRequest,
    ) -> SendLocalListResponse {
        let status = self.local_list.update(request, now);
        if status == SendLocalListStatus::Accepted {
            let kind = EventKind::LocalListUpdated {
                version: self.local_list.version,
                entries: self.local_list.tokens.len(),
            };
            self.emit(out, now, kind);
        }
        SendLocalListResponse {
            status,
            status_info: None,
        }
    }

    pub(super) fn local_list_version(&self) -> GetLocalListVersionResponse {
        let version_number = match self.flag(LOCAL_AUTH_LIST_CTRLR, ENABLED) {
            true => self.local_list.version,
            false => 0,
        };
        GetLocalListVersionResponse { version_number }
    }

    fn auth_flag(&self, variable: &str) -> bool {
        self.flag(AUTH_CTRLR, variable)
    }

    fn flag(&self, component: &str, variable: &str) -> bool {
        self.device_model.value(component, variable) == Some("true")
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
