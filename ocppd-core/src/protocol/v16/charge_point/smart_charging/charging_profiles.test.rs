use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::charging_profiles::{clear, profile, set};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::protocol::v16::event::EventKind;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn tx_profile_for(transaction_id: i32) -> Value {
    let mut tx = profile(1, "TxProfile", 0);
    tx["transactionId"] = json!(transaction_id);
    tx
}

fn twenty_profiles() -> Csms {
    let mut csms = Csms::booted();
    for id in 0..20 {
        let level = id as u32 % 11;
        let connector = id as u32 / 11;
        set(&mut csms, connector, profile(id, "TxDefaultProfile", level));
    }
    csms
}

fn fine() -> Value {
    let mut fine = profile(1, "TxDefaultProfile", 0);
    fine["chargingSchedule"]["chargingSchedulePeriod"][0]["limit"] = json!(16.3);
    fine["chargingSchedule"]["minChargingRate"] = json!(6.1);
    fine
}

#[test]
fn given_booted_charge_point_when_charge_point_max_profile_set_on_connector_1_then_rejected() {
    let mut csms = Csms::booted();

    let status = set(&mut csms, 1, profile(1, "ChargePointMaxProfile", 0));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_charge_point_when_charge_point_max_profile_set_on_connector_0_then_accepted_and_reported()
 {
    let mut csms = Csms::booted();

    let status = set(&mut csms, 0, profile(1, "ChargePointMaxProfile", 0));

    assert_eq!(status, "Accepted");
    assert!(csms.events.contains(&EventKind::ChargingProfileSet {
        connector: 0,
        charging_profile_id: 1,
    }));
}

#[test]
fn given_one_connector_when_tx_default_profiles_set_on_connectors_0_1_and_2_then_only_known_ones_accepted()
 {
    let mut csms = Csms::booted();

    let statuses = [
        set(&mut csms, 0, profile(1, "TxDefaultProfile", 0)),
        set(&mut csms, 1, profile(2, "TxDefaultProfile", 0)),
        set(&mut csms, 2, profile(3, "TxDefaultProfile", 0)),
    ];

    assert_eq!(
        statuses,
        [json!("Accepted"), json!("Accepted"), json!("Rejected")]
    );
}

#[test]
fn given_no_transaction_when_tx_profile_set_then_rejected() {
    let mut csms = Csms::booted();

    let status = set(&mut csms, 1, profile(1, "TxProfile", 0));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_transaction_42_when_tx_profile_for_7_set_then_rejected() {
    let mut csms = Csms::charging(0, EvSpec::default());

    let status = set(&mut csms, 1, tx_profile_for(7));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_transaction_42_when_tx_profile_for_42_set_then_accepted() {
    let mut csms = Csms::charging(0, EvSpec::default());

    let status = set(&mut csms, 1, tx_profile_for(42));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_transaction_42_when_tx_profile_without_transaction_id_set_then_accepted() {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(&mut csms, 1, tx_profile_for(42));

    let status = set(&mut csms, 1, profile(2, "TxProfile", 1));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_booted_charge_point_when_stack_level_11_set_then_rejected() {
    let mut csms = Csms::booted();

    let status = set(&mut csms, 0, profile(1, "TxDefaultProfile", 11));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_charge_point_when_25_periods_set_then_rejected() {
    let mut csms = Csms::booted();
    let mut long = profile(1, "TxDefaultProfile", 0);
    let periods: Vec<_> = (0..25)
        .map(|n| json!({"startPeriod": n * 60, "limit": 16.0}))
        .collect();
    long["chargingSchedule"]["chargingSchedulePeriod"] = json!(periods);

    let status = set(&mut csms, 0, long);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_charge_point_when_20_profiles_set_then_all_accepted() {
    let mut csms = Csms::booted();

    let statuses: Vec<Value> = (0..20)
        .map(|id| {
            let level = id as u32 % 11;
            let connector = id as u32 / 11;
            set(&mut csms, connector, profile(id, "TxDefaultProfile", level))
        })
        .collect();

    assert_eq!(statuses, vec![json!("Accepted"); 20]);
}

#[test]
fn given_20_profiles_when_another_set_then_rejected() {
    let mut csms = twenty_profiles();

    let status = set(&mut csms, 1, profile(99, "TxDefaultProfile", 10));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_booted_charge_point_when_rates_in_tenths_set_then_accepted() {
    let mut csms = Csms::booted();

    let status = set(&mut csms, 1, fine());

    assert_eq!(status, "Accepted");
}

#[test]
fn given_booted_charge_point_when_rates_finer_than_a_tenth_set_then_property_constraint_violation()
{
    for (field, rate) in [("limit", 16.25), ("minChargingRate", 0.05)] {
        let mut csms = Csms::booted();
        let mut off = fine();
        match field {
            "limit" => off["chargingSchedule"]["chargingSchedulePeriod"][0]["limit"] = json!(rate),
            _ => off["chargingSchedule"]["minChargingRate"] = json!(rate),
        }
        let request = json!({"connectorId": 1, "csChargingProfiles": off});

        let reply = csms.call(0, "SetChargingProfile", request);

        match reply {
            Frame::CallError { code, .. } => assert_eq!(code, "PropertyConstraintViolation"),
            other => panic!("{field} {rate}: expected an error, got {other:?}"),
        }
    }
}

fn replaced_by_id() -> Csms {
    let mut csms = Csms::booted();
    set(&mut csms, 1, profile(1, "TxDefaultProfile", 0));
    set(&mut csms, 1, profile(2, "TxDefaultProfile", 0));
    csms
}

fn three_profiles() -> Csms {
    let mut csms = Csms::booted();
    set(&mut csms, 0, profile(1, "ChargePointMaxProfile", 0));
    set(&mut csms, 0, profile(2, "TxDefaultProfile", 0));
    set(&mut csms, 1, profile(3, "TxDefaultProfile", 1));
    csms
}

#[test]
fn given_tx_profile_when_its_transaction_stops_then_the_profile_is_gone() {
    let mut csms = Csms::charging(0, EvSpec::default());
    set(&mut csms, 1, profile(1, "TxProfile", 0));

    csms.intent(MINUTE, Intent::Stop { connector: 1 });

    assert_eq!(clear(&mut csms, json!({"id": 1})), "Unknown");
}

#[test]
fn given_profile_in_the_same_slot_when_cleared_by_the_old_id_then_unknown() {
    let mut csms = replaced_by_id();

    let status = clear(&mut csms, json!({"id": 1}));

    assert_eq!(status, "Unknown");
}

#[test]
fn given_profile_2_moved_to_connector_0_when_connector_1_cleared_then_unknown() {
    let mut csms = replaced_by_id();
    set(&mut csms, 0, profile(2, "ChargePointMaxProfile", 3));

    let status = clear(&mut csms, json!({"connectorId": 1}));

    assert_eq!(status, "Unknown");
}

#[test]
fn given_profile_2_moved_to_connector_0_when_cleared_by_id_then_accepted() {
    let mut csms = replaced_by_id();
    set(&mut csms, 0, profile(2, "ChargePointMaxProfile", 3));

    let status = clear(&mut csms, json!({"id": 2}));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_profiles_when_cleared_by_a_purpose_and_an_unused_stack_level_then_unknown() {
    let mut csms = three_profiles();
    let request = json!({"chargingProfilePurpose": "TxDefaultProfile", "stackLevel": 5});

    let status = clear(&mut csms, request);

    assert_eq!(status, "Unknown");
}

#[test]
fn given_profiles_when_cleared_by_connector_and_purpose_then_the_one_matching_both_cleared() {
    let mut csms = three_profiles();
    let request = json!({"connectorId": 0, "chargingProfilePurpose": "TxDefaultProfile"});

    let status = clear(&mut csms, request);

    assert_eq!(status, "Accepted");
    assert!(
        csms.events
            .contains(&EventKind::ChargingProfilesCleared { count: 1 })
    );
}

#[test]
fn given_two_profiles_left_when_cleared_without_criteria_then_both_cleared() {
    let mut csms = three_profiles();
    let request = json!({"connectorId": 0, "chargingProfilePurpose": "TxDefaultProfile"});
    clear(&mut csms, request);

    let status = clear(&mut csms, json!({}));

    assert_eq!(status, "Accepted");
    assert!(
        csms.events
            .contains(&EventKind::ChargingProfilesCleared { count: 2 })
    );
}
