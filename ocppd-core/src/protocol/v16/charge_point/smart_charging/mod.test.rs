use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::charging_profiles::status;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize, register};
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use serde_json::{Value, json};

const EIGHT_AMPS_W: f64 = 8.0 * 230.0 * 3.0;

fn tx_profile(periods: &[(u32, f64)]) -> Value {
    let periods: Vec<_> = periods
        .iter()
        .map(|&(start, limit)| json!({"startPeriod": start, "limit": limit}))
        .collect();
    json!({
        "chargingProfileId": 1,
        "stackLevel": 0,
        "chargingProfilePurpose": "TxProfile",
        "chargingProfileKind": "Relative",
        "chargingSchedule": {"chargingRateUnit": "A", "chargingSchedulePeriod": periods},
    })
}

fn set(csms: &mut Csms, now: u64, connector: u32, profile: Value) {
    let request = json!({"connectorId": connector, "csChargingProfiles": profile});
    assert_eq!(
        status(csms.call(now, "SetChargingProfile", request)),
        "Accepted"
    );
}

fn power(meter_values: &Value) -> f64 {
    let sampled = meter_values["meterValue"][0]["sampledValue"]
        .as_array()
        .unwrap();
    let power = sampled
        .iter()
        .find(|value| value["measurand"] == "Power.Active.Import")
        .unwrap();
    power["value"].as_str().unwrap().parse().unwrap()
}

fn limits(csms: &Csms) -> Vec<f64> {
    csms.events
        .iter()
        .filter_map(|kind| match kind {
            EventKind::ChargingLimitChanged { limit_w, .. } => Some(*limit_w),
            _ => None,
        })
        .collect()
}

fn limited(periods: &[(u32, f64)]) -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(&mut csms, MINUTE, 1, tx_profile(periods));
    csms
}

fn stepping_down() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(&mut csms, 0, 1, tx_profile(&[(0, 16.0), (90, 8.0)]));
    csms.input(MINUTE, Input::Tick);
    csms
}

fn remote_start(csms: &mut Csms, profile: Value) -> Value {
    let request = json!({"connectorId": 1, "idTag": "TAG1", "chargingProfile": profile});
    status(csms.call(0, "RemoteStartTransaction", request))
}

fn plugged() -> Csms {
    let mut csms = Csms::booted();
    let ev = EvSpec::default();
    csms.intent(0, Intent::PlugIn { connector: 1, ev });
    csms
}

fn started(csms: &mut Csms, now: u64) {
    csms.answer(
        now,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 42}),
    );
}

fn default_profile() -> Value {
    let mut default = tx_profile(&[(0, 8.0)]);
    default["chargingProfilePurpose"] = json!("TxDefaultProfile");
    default
}

#[test]
fn given_charging_ev_when_tx_profile_set_then_limit_lowered_at_once() {
    let mut csms = Csms::charging(0, EvSpec::default());

    set(&mut csms, MINUTE, 1, tx_profile(&[(0, 8.0)]));

    assert_eq!(limits(&csms), vec![EIGHT_AMPS_W]);
}

#[test]
fn given_lowered_limit_when_next_meter_values_sent_then_the_lower_power_is_metered() {
    let mut csms = limited(&[(0, 8.0)]);

    csms.input(2 * MINUTE, Input::Tick);

    let second = &csms.sent("MeterValues")[1];
    assert_eq!(power(second), EIGHT_AMPS_W);
    assert_eq!(register(second), 275.0);
}

#[test]
fn given_schedule_stepping_down_at_90_seconds_when_ticked_before_then_wakes_up_at_the_step() {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(&mut csms, 0, 1, tx_profile(&[(0, 16.0), (90, 8.0)]));

    csms.input(MINUTE, Input::Tick);

    assert_eq!(csms.charge_point.next_deadline(), Some(90_000));
}

#[test]
fn given_schedule_stepping_down_when_the_step_comes_then_limit_lowered() {
    let mut csms = stepping_down();

    csms.input(90_000, Input::Tick);

    assert_eq!(limits(&csms).last(), Some(&EIGHT_AMPS_W));
}

#[test]
fn given_stepped_down_schedule_when_next_meter_values_sent_then_the_lower_power_is_metered() {
    let mut csms = stepping_down();
    csms.input(90_000, Input::Tick);

    csms.input(2 * MINUTE, Input::Tick);

    assert_eq!(power(&csms.sent("MeterValues")[1]), EIGHT_AMPS_W);
}

#[test]
fn given_charging_ev_when_zero_limit_set_then_suspended_evse() {
    let mut csms = Csms::charging(0, EvSpec::default());

    set(&mut csms, MINUTE, 1, tx_profile(&[(0, 0.0)]));

    assert_eq!(
        csms.statuses().last(),
        Some(&ChargePointStatus::SuspendedEvse)
    );
}

#[test]
fn given_zero_limit_when_profile_cleared_then_charging_at_the_hardware_limit() {
    let mut csms = limited(&[(0, 0.0)]);

    csms.call(MINUTE, "ClearChargingProfile", json!({"id": 1}));

    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Charging));
    assert_eq!(limits(&csms), vec![0.0, 22_080.0]);
}

#[test]
fn given_tx_default_profile_set_before_the_transaction_when_meter_values_sent_then_limited_from_the_start()
 {
    let mut csms = Csms::booted();
    set(&mut csms, 0, 0, default_profile());
    let ev = EvSpec::default();
    csms.intent(0, Intent::PlugIn { connector: 1, ev });
    csms.intent(0, authorize("TAG1"));
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));
    started(&mut csms, 0);

    csms.input(MINUTE, Input::Tick);

    assert_eq!(power(&csms.sent("MeterValues")[0]), EIGHT_AMPS_W);
    assert!(limits(&csms).is_empty());
}

#[test]
fn given_plugged_ev_when_remote_start_with_tx_profile_then_accepted() {
    let mut csms = plugged();

    let status = remote_start(&mut csms, tx_profile(&[(0, 8.0)]));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_remote_start_with_profile_when_transaction_started_then_it_becomes_the_transaction_profile()
 {
    let mut csms = plugged();
    remote_start(&mut csms, tx_profile(&[(0, 8.0)]));

    started(&mut csms, 0);

    assert!(csms.events.contains(&EventKind::ChargingProfileSet {
        connector: 1,
        charging_profile_id: 1,
    }));
}

#[test]
fn given_remote_starts_authorized_first_when_a_remote_start_with_profile_is_authorized_and_started_then_it_becomes_the_transaction_profile()
 {
    let mut csms = plugged();
    csms.configure(0, "AuthorizeRemoteTxRequests", "true");
    remote_start(&mut csms, tx_profile(&[(0, 8.0)]));
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    started(&mut csms, 0);

    assert!(csms.events.contains(&EventKind::ChargingProfileSet {
        connector: 1,
        charging_profile_id: 1,
    }));
}

#[test]
fn given_transaction_started_with_a_remote_profile_when_meter_values_sent_then_limited() {
    let mut csms = plugged();
    remote_start(&mut csms, tx_profile(&[(0, 8.0)]));
    started(&mut csms, 0);

    csms.input(MINUTE, Input::Tick);

    assert_eq!(power(&csms.sent("MeterValues")[0]), EIGHT_AMPS_W);
}

#[test]
fn given_no_ev_when_remote_start_with_tx_profile_then_accepted() {
    let mut csms = Csms::booted();

    let status = remote_start(&mut csms, tx_profile(&[(0, 8.0)]));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_remote_start_with_profile_before_the_cable_when_meter_values_sent_then_limited() {
    let mut csms = Csms::booted();
    remote_start(&mut csms, tx_profile(&[(0, 8.0)]));
    let ev = EvSpec::default();
    csms.intent(10_000, Intent::PlugIn { connector: 1, ev });
    started(&mut csms, 10_000);

    csms.input(10_000 + MINUTE, Input::Tick);

    assert_eq!(power(&csms.sent("MeterValues")[0]), EIGHT_AMPS_W);
}

#[test]
fn given_booted_charge_point_when_remote_start_with_tx_default_profile_then_rejected() {
    let mut csms = Csms::booted();

    let status = remote_start(&mut csms, default_profile());

    assert_eq!(status, "Rejected");
}
