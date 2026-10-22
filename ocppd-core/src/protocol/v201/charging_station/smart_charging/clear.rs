use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::smart_charging::clear::{
    ClearChargingProfileRequest, ClearChargingProfileResponse, ClearChargingProfileStatus,
};

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn clear_charging_profile(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: ClearChargingProfileRequest,
    ) -> ClearChargingProfileResponse {
        let count = self.charging_profiles.clear(&request);
        let status = match count {
            0 => ClearChargingProfileStatus::Unknown,
            _ => ClearChargingProfileStatus::Accepted,
        };
        if count > 0 {
            self.count_profiles();
            self.emit(out, now, EventKind::ChargingProfilesCleared { count });
        }
        ClearChargingProfileResponse {
            status,
            status_info: None,
        }
    }
}

#[cfg(test)]
#[path = "clear.test.rs"]
mod tests;
