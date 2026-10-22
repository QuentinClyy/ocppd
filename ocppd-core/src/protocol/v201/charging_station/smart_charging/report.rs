use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::messages::smart_charging::report::{
    ChargingLimitSource, GetChargingProfileStatus, GetChargingProfilesRequest,
    GetChargingProfilesResponse, ReportChargingProfilesRequest,
};
use serde_json::json;

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn get_charging_profiles(
        &mut self,
        request: GetChargingProfilesRequest,
    ) -> GetChargingProfilesResponse {
        let mut matching = self
            .charging_profiles
            .matching(request.evse_id, &request.charging_profile);
        matching.sort_by_key(|(evse, _)| *evse);
        let mut reports: Vec<ReportChargingProfilesRequest> = Vec::new();
        for (evse, profile) in matching {
            match reports.last_mut() {
                Some(report) if report.evse_id == evse => {
                    report.charging_profile.push(profile.clone())
                }
                _ => reports.push(ReportChargingProfilesRequest {
                    request_id: request.request_id,
                    charging_limit_source: ChargingLimitSource::Cso,
                    charging_profile: vec![profile.clone()],
                    tbc: true,
                    evse_id: evse,
                }),
            }
        }
        let status = match reports.is_empty() {
            true => GetChargingProfileStatus::NoProfiles,
            false => GetChargingProfileStatus::Accepted,
        };
        if let Some(last) = reports.last_mut() {
            last.tbc = false;
        }
        for report in reports {
            self.outbox.push("ReportChargingProfiles", 0, json!(report));
        }
        GetChargingProfilesResponse {
            status,
            status_info: None,
        }
    }
}
