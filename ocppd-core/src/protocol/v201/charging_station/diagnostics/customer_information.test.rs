use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

const CARD: &str = "04A2B3C4";

fn card() -> Value {
    json!({"idToken": CARD, "type": "ISO14443"})
}

fn cached(run: &mut Run) {
    let intent = Intent::Authorize {
        connector: Some(1),
        id_tag: CARD.into(),
    };
    run.input(0, Input::Intent(intent));
    run.answer(0, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(0);
    run.calls.clear();
}

fn ask(run: &mut Run, request: Value) -> Value {
    let status = run.result(0, "CustomerInformation", request)["status"].clone();
    run.answer_all(0);
    status
}

fn by_card(report: bool, clear: bool) -> Value {
    json!({"requestId": 5, "report": report, "clear": clear, "idToken": card()})
}

fn notified(run: &Run) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, action, _)| action == "NotifyCustomerInformation")
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

fn data(run: &Run) -> Vec<Value> {
    notified(run)
        .iter()
        .map(|notify| notify["data"].clone())
        .collect()
}

fn listed_blocked_and_cached() -> Run {
    let mut run = Run::accepted(86_400);
    let entry = json!({"idToken": card(), "idTokenInfo": {"status": "Blocked"}});
    let list = json!({"versionNumber": 1, "updateType": "Full", "localAuthorizationList": [entry]});
    run.result(0, "SendLocalList", list);
    cached(&mut run);
    run
}

fn first_data(run: &Run) -> Value {
    serde_json::from_str(data(run)[0].as_str().unwrap()).unwrap()
}

#[test]
fn given_cached_id_token_when_its_data_reported_then_one_notification_with_the_cache_entry_per_n09_fr_05_and_tc_n_27()
 {
    let mut run = Run::accepted(86_400);
    cached(&mut run);

    let status = ask(&mut run, by_card(true, false));

    assert_eq!(status, "Accepted");
    let notify = notified(&run).pop().unwrap();
    assert_eq!(notify["requestId"], 5);
    assert_eq!(
        (notify["seqNo"].clone(), notify["tbc"].clone()),
        (json!(0), json!(false))
    );
    let reported: Value = serde_json::from_str(notify["data"].as_str().unwrap()).unwrap();
    assert_eq!(
        reported,
        json!({"authorizationCache": {"status": "Accepted"}})
    );
}

#[test]
fn given_unknown_id_token_when_its_data_reported_then_one_empty_report_per_n09_fr_06_and_tc_n_28() {
    let mut run = Run::accepted(86_400);

    let status = ask(&mut run, by_card(true, false));

    assert_eq!(status, "Accepted");
    assert_eq!(data(&run), vec![""]);
}

#[test]
fn given_unknown_customer_identifier_when_its_data_reported_then_one_empty_report_per_n09_fr_06() {
    let mut run = Run::accepted(86_400);
    let by_identifier =
        json!({"requestId": 5, "report": true, "clear": false, "customerIdentifier": "C-1"});

    ask(&mut run, by_identifier);

    assert_eq!(data(&run), vec![""]);
}

#[test]
fn given_booted_station_when_data_asked_without_customer_reference_then_invalid_per_n09_fr_09() {
    let mut run = Run::accepted(86_400);

    let status = ask(
        &mut run,
        json!({"requestId": 5, "report": true, "clear": false}),
    );

    assert_eq!(status, "Invalid");
    assert!(notified(&run).is_empty());
}

#[test]
fn given_booted_station_when_data_asked_with_two_customer_references_then_invalid_per_n09_fr_09() {
    let mut run = Run::accepted(86_400);
    let mut two = by_card(true, false);
    two["customerIdentifier"] = json!("C-1");

    let status = ask(&mut run, two);

    assert_eq!(status, "Invalid");
    assert!(notified(&run).is_empty());
}

#[test]
fn given_booted_station_when_neither_report_nor_clear_asked_then_rejected_per_n09_fr_07_and_tc_n_29()
 {
    let mut run = Run::accepted(86_400);

    let status = ask(&mut run, by_card(false, false));

    assert_eq!(status, "Rejected");
    assert!(notified(&run).is_empty());
}

#[test]
fn given_listed_and_cached_id_token_when_cleared_with_report_then_both_reported_per_n10_fr_03_and_tc_n_30()
 {
    let mut run = listed_blocked_and_cached();

    let status = ask(&mut run, by_card(true, true));

    assert_eq!(status, "Accepted");
    let reported = first_data(&run);
    assert_eq!(reported["authorizationCache"]["status"], "Accepted");
    assert_eq!(reported["localList"]["status"], "Blocked");
}

#[test]
fn given_cleared_customer_when_reported_again_then_only_the_local_list_left_per_n10_fr_03() {
    let mut run = listed_blocked_and_cached();
    ask(&mut run, by_card(true, true));
    run.calls.clear();

    ask(&mut run, by_card(true, false));

    assert_eq!(
        first_data(&run),
        json!({"localList": {"status": "Blocked"}})
    );
}

#[test]
fn given_cached_id_token_when_cleared_without_report_then_cleared_notified_per_n10_fr_06_and_tc_n_32()
 {
    let mut run = Run::accepted(86_400);
    cached(&mut run);

    let status = ask(&mut run, by_card(false, true));

    assert_eq!(status, "Accepted");
    assert_eq!(data(&run), vec!["Cleared"]);
}

#[test]
fn given_cleared_cached_id_token_when_reported_then_empty_per_n10_fr_06_and_tc_n_31() {
    let mut run = Run::accepted(86_400);
    cached(&mut run);
    ask(&mut run, by_card(false, true));
    run.calls.clear();

    ask(&mut run, by_card(true, false));

    assert_eq!(data(&run), vec![""]);
}

#[test]
fn given_long_personal_message_in_the_list_when_reported_then_split_into_parts() {
    let mut run = Run::accepted(86_400);
    let message = json!({"format": "UTF8", "content": "x".repeat(500)});
    let entry = json!({"idToken": card(), "idTokenInfo": {"status": "Accepted", "personalMessage": message}});
    let list = json!({"versionNumber": 1, "updateType": "Full", "localAuthorizationList": [entry]});
    run.result(0, "SendLocalList", list);

    ask(&mut run, by_card(true, false));

    let parts = notified(&run);
    let flags: Vec<_> = parts
        .iter()
        .map(|part| (part["seqNo"].clone(), part["tbc"].clone()))
        .collect();
    assert_eq!(
        flags,
        vec![(json!(0), json!(true)), (json!(1), json!(false))]
    );
    let joined: String = parts
        .iter()
        .map(|part| part["data"].as_str().unwrap())
        .collect();
    let reported: Value = serde_json::from_str(&joined).unwrap();
    assert_eq!(
        reported["localList"]["personalMessage"]["content"],
        "x".repeat(500)
    );
}
