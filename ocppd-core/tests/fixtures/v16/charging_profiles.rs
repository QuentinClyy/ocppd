use crate::fixtures::v16::fake_csms::Csms;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

pub fn status(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { payload, .. } => payload["status"].clone(),
        other => panic!("expected a result, got {other:?}"),
    }
}

pub fn profile(id: i32, purpose: &str, stack_level: u32) -> Value {
    json!({
        "chargingProfileId": id,
        "stackLevel": stack_level,
        "chargingProfilePurpose": purpose,
        "chargingProfileKind": "Absolute",
        "chargingSchedule": {
            "chargingRateUnit": "A",
            "chargingSchedulePeriod": [{"startPeriod": 0, "limit": 16.0}],
        },
    })
}

pub fn set(csms: &mut Csms, connector: u32, profile: Value) -> Value {
    let request = json!({"connectorId": connector, "csChargingProfiles": profile});
    status(csms.call(0, "SetChargingProfile", request))
}

pub fn clear(csms: &mut Csms, request: Value) -> Value {
    status(csms.call(0, "ClearChargingProfile", request))
}
