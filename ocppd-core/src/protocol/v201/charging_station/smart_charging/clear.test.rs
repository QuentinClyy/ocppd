use crate::fixtures::v201::charging_profiles::{
    AMPS_TO_W, HARDWARE_W, absolute, charging, entries, offered_w, profile, set,
};
use crate::fixtures::v201::station::Run;
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

fn installed() -> Run {
    let mut run = Run::accepted_with(2, 86_400);
    set(&mut run, 0, 1, profile(1, "TxDefaultProfile", &[(0, 10.0)]));
    set(&mut run, 0, 2, profile(2, "TxDefaultProfile", &[(0, 12.0)]));
    let max = absolute(profile(3, "ChargingStationMaxProfile", &[(0, 30.0)]));
    set(&mut run, 0, 0, max);
    run.calls.clear();
    run
}

fn get(run: &mut Run, evse: Option<u32>, criterion: Value) -> Value {
    let mut request = json!({"requestId": 5, "chargingProfile": criterion});
    if let Some(evse) = evse {
        request["evseId"] = json!(evse);
    }
    let status = run.result(0, "GetChargingProfiles", request)["status"].clone();
    run.answer_all(0);
    status
}

fn reports(run: &Run) -> Vec<(u64, Vec<i64>, bool)> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "ReportChargingProfiles")
        .map(|(_, _, report)| {
            assert_eq!(report["requestId"], 5, "K09.FR.01");
            assert_eq!(report["chargingLimitSource"], "CSO");
            let ids = report["chargingProfile"].as_array().unwrap();
            let ids = ids.iter().map(|profile| profile["id"].as_i64().unwrap());
            let evse = report["evseId"].as_u64().unwrap();
            (evse, ids.collect(), report["tbc"].as_bool().unwrap())
        })
        .collect()
}

fn clear(run: &mut Run, request: Value) -> Value {
    run.result(0, "ClearChargingProfile", request)["status"].clone()
}

#[test]
fn given_profiles_on_evse_0_1_and_2_when_all_asked_then_reported_per_evse_with_tbc_until_the_last_per_k09_fr_01_fr_02_fr_06_and_tc_k_31()
 {
    let mut run = installed();

    let status = get(&mut run, None, json!({}));

    assert_eq!(status, "Accepted");
    assert_eq!(
        reports(&run),
        vec![(0, vec![3], true), (1, vec![1], true), (2, vec![2], false)]
    );
}

#[test]
fn given_profiles_on_several_evses_when_evse_1_asked_then_only_its_profiles_per_k09_fr_04_and_tc_k_30()
 {
    let mut run = installed();

    get(&mut run, Some(1), json!({}));

    assert_eq!(reports(&run), vec![(1, vec![1], false)]);
}

#[test]
fn given_profiles_on_several_evses_when_evse_0_asked_then_only_the_station_profiles_per_k09_fr_05_and_tc_k_29()
 {
    let mut run = installed();

    get(&mut run, Some(0), json!({}));

    assert_eq!(reports(&run), vec![(0, vec![3], false)]);
}

#[test]
fn given_installed_profiles_when_asked_by_profile_id_then_only_those_reported_per_k09_fr_03_and_tc_k_32()
 {
    let mut run = installed();

    get(&mut run, None, json!({"chargingProfileId": [2, 3]}));

    assert_eq!(reports(&run), vec![(0, vec![3], true), (2, vec![2], false)]);
}

#[test]
fn given_installed_profiles_when_asked_by_purpose_and_stack_level_then_only_matches_reported_per_k09_fr_03_and_tc_k_36()
 {
    let mut run = installed();
    let purpose = json!({"chargingProfilePurpose": "TxDefaultProfile", "stackLevel": 0});

    get(&mut run, None, purpose);

    assert_eq!(reports(&run), vec![(1, vec![1], true), (2, vec![2], false)]);
}

#[test]
fn given_cso_profiles_when_asked_for_another_limit_source_then_no_profiles_per_tc_k_34() {
    let mut run = installed();

    let status = get(&mut run, None, json!({"chargingLimitSource": ["EMS"]}));

    assert_eq!(status, "NoProfiles");
    assert_eq!(reports(&run), vec![]);
}

#[test]
fn given_profiles_at_stack_level_0_when_asked_for_stack_level_4_then_no_profiles() {
    let mut run = installed();

    let status = get(&mut run, None, json!({"stackLevel": 4}));

    assert_eq!(status, "NoProfiles");
}

#[test]
fn given_installed_profiles_when_unknown_id_cleared_then_unknown_per_k10_fr_01_and_tc_k_08() {
    let mut run = installed();

    let status = clear(&mut run, json!({"chargingProfileId": 9}));

    assert_eq!(status, "Unknown");
}

#[test]
fn given_installed_profiles_when_one_cleared_by_id_then_removed_per_k10_fr_03_and_tc_k_05() {
    let mut run = installed();

    let status = clear(&mut run, json!({"chargingProfileId": 2}));

    assert_eq!(status, "Accepted");
    assert_eq!(entries(&mut run), "2");
    assert!(
        run.events
            .contains(&EventKind::ChargingProfilesCleared { count: 1 })
    );
}

#[test]
fn given_cleared_profile_when_cleared_again_then_unknown_per_k10_fr_01() {
    let mut run = installed();
    clear(&mut run, json!({"chargingProfileId": 2}));

    let status = clear(&mut run, json!({"chargingProfileId": 2}));

    assert_eq!(status, "Unknown");
}

#[test]
fn given_installed_profiles_when_cleared_by_purpose_then_every_match_removed_per_k10_fr_04_and_tc_k_06()
 {
    let mut run = installed();
    let criteria =
        json!({"chargingProfileCriteria": {"chargingProfilePurpose": "TxDefaultProfile"}});

    let status = clear(&mut run, criteria);

    assert_eq!(status, "Accepted");
    assert_eq!(entries(&mut run), "1");
}

#[test]
fn given_defaults_cleared_when_evse_1_cleared_then_unknown_per_k10_fr_04_and_tc_k_07() {
    let mut run = installed();
    let criteria =
        json!({"chargingProfileCriteria": {"chargingProfilePurpose": "TxDefaultProfile"}});
    clear(&mut run, criteria);

    let status = clear(&mut run, json!({"chargingProfileCriteria": {"evseId": 1}}));

    assert_eq!(status, "Unknown");
}

#[test]
fn given_installed_profiles_when_ev_charges_on_evse_1_then_limited_by_its_default() {
    let mut run = installed();

    charging(&mut run, 0, 1);

    assert_eq!(offered_w(&run, 0), Some(10.0 * AMPS_TO_W));
}

#[test]
fn given_default_limited_transaction_when_its_default_cleared_then_limited_by_the_station_max_per_k10_fr_05_and_tc_k_09()
 {
    let mut run = installed();
    charging(&mut run, 0, 1);

    clear(&mut run, json!({"chargingProfileId": 1}));

    assert_eq!(offered_w(&run, 0), Some(30.0 * AMPS_TO_W));
}

#[test]
fn given_only_the_station_max_left_when_it_is_cleared_then_hardware_limit_per_k10_fr_07() {
    let mut run = installed();
    charging(&mut run, 0, 1);
    clear(&mut run, json!({"chargingProfileId": 1}));

    clear(&mut run, json!({"chargingProfileId": 3}));

    assert_eq!(offered_w(&run, 0), Some(HARDWARE_W));
}
