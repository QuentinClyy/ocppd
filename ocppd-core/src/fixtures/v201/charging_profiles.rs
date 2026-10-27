use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::{Value, json};

pub const AMPS_TO_W: f64 = 230.0 * 3.0;
pub const HARDWARE_W: f64 = 32.0 * AMPS_TO_W;
pub const HALF_MINUTE: u64 = MINUTE / 2;

pub fn charging(run: &mut Run, now: u64, connector: u32) -> String {
    let ev = EvSpec::default();
    run.input(now, Input::Intent(Intent::PlugIn { connector, ev }));
    run.answer_all(now);
    let intent = Intent::Authorize {
        connector: Some(connector),
        id_tag: "04A2B3C4".into(),
    };
    run.input(now, Input::Intent(intent));
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(now);
    let started = run
        .calls
        .iter()
        .rev()
        .find(|(_, action, payload)| {
            action == "TransactionEvent" && payload["eventType"] == "Started"
        })
        .expect("no transaction started");
    started.2["transactionInfo"]["transactionId"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub fn profile(id: i32, purpose: &str, periods: &[(u32, f64)]) -> Value {
    let periods: Vec<_> = periods
        .iter()
        .map(|&(start, limit)| json!({"startPeriod": start, "limit": limit}))
        .collect();
    json!({
        "id": id,
        "stackLevel": 0,
        "chargingProfilePurpose": purpose,
        "chargingProfileKind": "Relative",
        "chargingSchedule": [{"id": id, "chargingRateUnit": "A", "chargingSchedulePeriod": periods}],
    })
}

pub fn tx_profile(id: i32, transaction_id: &str, periods: &[(u32, f64)]) -> Value {
    let mut profile = profile(id, "TxProfile", periods);
    profile["transactionId"] = json!(transaction_id);
    profile
}

pub fn absolute(mut profile: Value) -> Value {
    profile["chargingProfileKind"] = json!("Absolute");
    profile["chargingSchedule"][0]["startSchedule"] = json!("1970-01-01T00:00:00Z");
    profile
}

pub fn set(run: &mut Run, now: u64, evse: u32, profile: Value) -> Value {
    let request = json!({"evseId": evse, "chargingProfile": profile});
    run.result(now, "SetChargingProfile", request)
}

pub fn reason_code(run: &mut Run, evse: u32, profile: Value) -> Value {
    set(run, 0, evse, profile)["statusInfo"]["reasonCode"].clone()
}

pub fn offered_w(run: &Run, connector: usize) -> Option<f64> {
    run.station.station().connectors[connector].offered_w()
}

pub fn entries(run: &mut Run) -> Value {
    let request = json!({"getVariableData": [{"component": {"name": "SmartChargingCtrlr"}, "variable": {"name": "Entries", "instance": "ChargingProfiles"}}]});
    run.result(0, "GetVariables", request)["getVariableResult"][0]["attributeValue"].clone()
}
