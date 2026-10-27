use super::*;
use crate::fixtures::v201::reliability::{
    charging, configure, events, expected, last, plug_in, swipe,
};
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::json;

#[test]
fn given_4_or_7_byte_hex_uids_when_typed_then_iso14443_per_3_43() {
    let uids = ["04A2B3C4", "04a2b3c4d5e6f7"];

    let types = uids.map(token_type);

    assert_eq!(types, [IdTokenType::Iso14443, IdTokenType::Iso14443]);
}

#[test]
fn given_8_byte_hex_uid_when_typed_then_iso15693_per_3_43() {
    let uid = "E004010203040506";

    let kind = token_type(uid);

    assert_eq!(kind, IdTokenType::Iso15693);
}

#[test]
fn given_other_texts_when_typed_then_local_per_3_43() {
    for local in ["TAG1", "04A2B3C", "04A2B3CG", ""] {
        let kind = token_type(local);

        assert_eq!(kind, IdTokenType::Local, "{local}");
    }
}

fn authorized_with_event_pending(run: &mut Run) {
    plug_in(run, 0);
    run.answer_all(0);
    swipe(run, 0, "04A2B3C4");
    run.answer(0, json!({"idTokenInfo": {"status": "Accepted"}}));
}

fn blocked(run: &mut Run) {
    run.answer(0, json!({"idTokenInfo": {"status": "Blocked"}}));
    run.answer_all(0);
}

fn master_pass(run: &mut Run) {
    configure(run, "AuthCtrlr.MasterPassGroupId", "POLICE");
}

fn police(run: &mut Run, now: u64) {
    run.answer(now, json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "POLICE", "type": "Central"}}}));
    run.answer_all(now);
}

#[test]
fn given_authorized_transaction_when_its_id_token_comes_back_blocked_then_ended_as_deauthorized_per_e05_fr_10()
 {
    let mut run = Run::accepted(86_400);
    authorized_with_event_pending(&mut run);

    blocked(&mut run);

    assert_eq!(
        last(&run),
        expected(
            "Ended",
            "Deauthorized",
            json!("EVConnected"),
            json!("DeAuthorized")
        )
    );
}

#[test]
fn given_ev_connected_stop_point_when_id_token_comes_back_blocked_then_only_deauthorized_per_e05_fr_09()
 {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "TxCtrlr.TxStopPoint", "EVConnected");
    authorized_with_event_pending(&mut run);

    blocked(&mut run);

    assert_eq!(
        last(&run),
        expected("Updated", "Deauthorized", json!("EVConnected"), json!(null))
    );
}

#[test]
fn given_no_stop_on_invalid_id_when_id_token_comes_back_blocked_then_energy_suspended_per_e05_fr_02()
 {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "TxCtrlr.StopTxOnInvalidId", "false");
    authorized_with_event_pending(&mut run);

    blocked(&mut run);

    assert_eq!(
        last(&run),
        expected(
            "Updated",
            "ChargingStateChanged",
            json!("SuspendedEVSE"),
            json!(null)
        )
    );
}

#[test]
fn given_charging_ev_when_master_pass_presented_then_authorize_sent_per_c16_fr_02() {
    let mut run = Run::accepted(86_400);
    master_pass(&mut run);
    charging(&mut run, 0);

    swipe(&mut run, MINUTE, "OFFICER1");

    assert_eq!(run.last_call().1, "Authorize");
}

#[test]
fn given_charging_ev_and_presented_master_pass_when_accepted_then_every_transaction_stopped_per_c16_fr_02_and_fr_08()
 {
    let mut run = Run::accepted(86_400);
    master_pass(&mut run);
    charging(&mut run, 0);
    swipe(&mut run, MINUTE, "OFFICER1");

    police(&mut run, MINUTE);

    assert_eq!(
        last(&run),
        expected(
            "Ended",
            "StopAuthorized",
            json!("EVConnected"),
            json!("MasterPass")
        )
    );
}

#[test]
fn given_authorized_start_point_and_presented_master_pass_when_accepted_then_no_transaction_per_c16_fr_03()
 {
    let mut run = Run::accepted(86_400);
    master_pass(&mut run);
    configure(&mut run, "TxCtrlr.TxStartPoint", "Authorized");
    swipe(&mut run, 0, "OFFICER1");

    police(&mut run, 0);

    assert!(events(&run).is_empty());
}

#[test]
fn given_charging_ev_when_id_token_of_the_same_group_accepted_then_stopped_with_its_id_token_per_e07_fr_02_and_tc_c_39()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, 0);
    swipe(&mut run, MINUTE, "04B5C6D7");

    run.answer(MINUTE, json!({"idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "FLEET", "type": "Central"}}}));
    run.answer_all(MINUTE);

    assert_eq!(
        events(&run).pop().unwrap()["idToken"],
        json!({"idToken": "04B5C6D7", "type": "ISO14443"})
    );
    assert_eq!(
        last(&run),
        expected(
            "Ended",
            "StopAuthorized",
            json!("EVConnected"),
            json!("Local")
        )
    );
}
