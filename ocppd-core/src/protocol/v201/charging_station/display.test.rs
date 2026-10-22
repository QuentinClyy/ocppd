use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::display::{count, get, ids, message, set};
use crate::fixtures::v201::station::{MINUTE, Run, boot};
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

fn timed() -> Run {
    let mut run = Run::accepted(300);
    let mut timed = message(1, "NormalCycle", "Happy hour");
    timed["endDateTime"] = json!("1970-01-01T00:01:00Z");
    set(&mut run, timed);
    run
}

fn ten_messages() -> Run {
    let mut run = Run::accepted(300);
    for id in 1..=10 {
        set(&mut run, message(id, "NormalCycle", "Hi"));
    }
    run
}

#[test]
fn given_booted_station_when_message_set_then_accepted_shown_and_counted_per_o01_and_tc_o_01() {
    let mut run = Run::accepted(300);

    let status = set(&mut run, message(1, "NormalCycle", "Welcome"));

    assert_eq!(status, "Accepted");
    assert!(run.events.contains(&EventKind::DisplayMessageSet {
        id: 1,
        priority: serde_json::from_value(json!("NormalCycle")).unwrap(),
        content: "Welcome".into(),
    }));
    assert_eq!(count(&mut run), "1");
}

#[test]
fn given_booted_station_when_html_message_set_then_format_not_supported_per_o01_fr_03_and_tc_o_19()
{
    let mut run = Run::accepted(300);
    let mut html = message(1, "NormalCycle", "<b>Hi</b>");
    html["message"]["format"] = json!("HTML");

    let status = set(&mut run, html);

    assert_eq!(status, "NotSupportedMessageFormat");
}

#[test]
fn given_one_display_when_message_set_for_another_display_then_rejected_per_o01_fr_22() {
    let mut run = Run::accepted(300);
    let mut screen = message(1, "NormalCycle", "Hi");
    screen["display"] = json!({"name": "Display", "instance": "Back"});

    let status = set(&mut run, screen);

    assert_eq!(status, "Rejected");
}

#[test]
fn given_no_transaction_when_message_set_for_a_transaction_then_unknown_transaction_per_o02_fr_01_and_tc_o_10()
 {
    let mut run = Run::accepted(300);
    let mut stranger = message(1, "NormalCycle", "Hi");
    stranger["transactionId"] = json!("tx-unknown");

    let status = set(&mut run, stranger);

    assert_eq!(status, "UnknownTransaction");
}

#[test]
fn given_ten_messages_when_an_eleventh_set_then_rejected_per_o01_fr_11() {
    let mut run = ten_messages();

    let status = set(&mut run, message(11, "NormalCycle", "Hi"));

    assert_eq!(status, "Rejected");
}

#[test]
fn given_ten_messages_when_one_replaced_then_accepted_per_o01_fr_11() {
    let mut run = ten_messages();

    let status = set(&mut run, message(10, "InFront", "Again"));

    assert_eq!(status, "Accepted", "a replacement fits");
}

#[test]
fn given_message_when_its_id_set_again_then_replaced_per_o06_fr_01_and_tc_o_12() {
    let mut run = Run::accepted(300);
    set(&mut run, message(1, "NormalCycle", "Welcome"));

    set(&mut run, message(1, "InFront", "Closed at 22:00"));

    let (_, messages) = get(&mut run, json!({}));
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["message"]["content"], "Closed at 22:00");
    assert_eq!(messages[0]["priority"], "InFront");
}

#[test]
fn given_always_front_message_when_another_always_front_set_then_it_replaces_the_old_one_per_o01_fr_16_and_tc_o_24()
 {
    let mut run = Run::accepted(300);
    set(&mut run, message(1, "AlwaysFront", "Out of order"));
    set(&mut run, message(2, "NormalCycle", "Welcome"));

    set(&mut run, message(3, "AlwaysFront", "Back soon"));

    let (_, messages) = get(&mut run, json!({}));
    assert_eq!(ids(&messages), vec![2, 3]);
    assert!(
        run.events
            .contains(&EventKind::DisplayMessageRemoved { id: 1 })
    );
}

#[test]
fn given_booted_station_when_timed_message_set_then_its_end_scheduled_per_o01_fr_07() {
    let run = timed();

    let deadline = run.station.next_deadline();

    assert_eq!(deadline, Some(MINUTE));
}

#[test]
fn given_timed_message_when_its_end_comes_then_removed_per_o01_fr_07_and_tc_o_14() {
    let mut run = timed();

    run.input(MINUTE, Input::Tick);

    assert!(
        run.events
            .contains(&EventKind::DisplayMessageRemoved { id: 1 })
    );
    assert_eq!(count(&mut run), "0");
}

#[test]
fn given_transaction_message_when_its_transaction_ends_then_removed_per_o02_fr_02() {
    let mut run = Run::accepted(300);
    let ev = EvSpec::default();
    run.input(0, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(0);
    let started = run
        .calls
        .iter()
        .find(|(_, action, _)| action == "TransactionEvent");
    let transaction_id = started.unwrap().2["transactionInfo"]["transactionId"].clone();
    let mut personal = message(1, "InFront", "Charging at 11 kW");
    personal["transactionId"] = transaction_id;
    set(&mut run, personal);

    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(MINUTE);

    assert!(
        run.events
            .contains(&EventKind::DisplayMessageRemoved { id: 1 })
    );
}

fn clear(run: &mut Run, id: i32) -> Value {
    run.result(0, "ClearDisplayMessage", json!({"id": id}))["status"].clone()
}

fn welcome_and_idle_card_only() -> Run {
    let mut run = Run::accepted(300);
    set(&mut run, message(1, "NormalCycle", "Welcome"));
    let mut idle = message(2, "InFront", "Card only");
    idle["state"] = json!("Idle");
    set(&mut run, idle);
    run
}

#[test]
fn given_no_messages_when_all_asked_then_unknown_per_o03_fr_06_and_tc_o_03() {
    let mut run = Run::accepted(300);

    let (status, _) = get(&mut run, json!({}));

    assert_eq!(status, "Unknown");
}

#[test]
fn given_two_messages_when_all_asked_then_both_reported_in_one_part_per_o03_fr_01_to_fr_05_and_tc_o_02()
 {
    let mut run = Run::accepted(300);
    set(&mut run, message(1, "NormalCycle", "Welcome"));
    set(&mut run, message(2, "InFront", "Card only"));

    let (status, messages) = get(&mut run, json!({}));

    assert_eq!(status, "Accepted");
    assert_eq!(ids(&messages), vec![1, 2]);
}

#[test]
fn given_two_messages_when_asked_by_ids_then_only_known_matches_per_o04_fr_01_and_tc_o_07() {
    let mut run = welcome_and_idle_card_only();

    let (_, messages) = get(&mut run, json!({"id": [2, 9]}));

    assert_eq!(ids(&messages), vec![2]);
}

#[test]
fn given_two_messages_when_asked_by_priority_then_only_matches_per_o04_fr_01_and_tc_o_08() {
    let mut run = welcome_and_idle_card_only();

    let (_, messages) = get(&mut run, json!({"priority": "NormalCycle"}));

    assert_eq!(ids(&messages), vec![1]);
}

#[test]
fn given_two_messages_when_asked_by_state_then_only_matches_per_o04_fr_01_and_tc_o_09() {
    let mut run = welcome_and_idle_card_only();

    let (_, messages) = get(&mut run, json!({"state": "Idle"}));

    assert_eq!(ids(&messages), vec![2]);
}

#[test]
fn given_two_messages_when_asked_for_an_unused_state_then_unknown_per_o04_fr_02_and_tc_o_34() {
    let mut run = welcome_and_idle_card_only();

    let (status, messages) = get(&mut run, json!({"state": "Charging"}));

    assert_eq!(status, "Unknown");
    assert!(messages.is_empty());
}

#[test]
fn given_message_when_cleared_then_accepted_and_gone_per_o05_fr_01_and_tc_o_04() {
    let mut run = Run::accepted(300);
    set(&mut run, message(1, "NormalCycle", "Welcome"));

    let status = clear(&mut run, 1);

    assert_eq!(status, "Accepted");
    assert_eq!(count(&mut run), "0");
}

#[test]
fn given_cleared_message_when_cleared_again_then_unknown_per_o05_fr_02_and_tc_o_05() {
    let mut run = Run::accepted(300);
    set(&mut run, message(1, "NormalCycle", "Welcome"));
    clear(&mut run, 1);

    let status = clear(&mut run, 1);

    assert_eq!(status, "Unknown");
}

#[test]
fn given_message_when_station_rebooted_then_still_reported_per_o01_fr_10_and_tc_o_20() {
    let mut run = welcome_and_idle_card_only();
    run.result(0, "Reset", json!({"type": "Immediate"}));
    run.answer_all(0);
    run.reconnect(MINUTE);
    run.answer(MINUTE, boot("Accepted", 300));
    run.answer_all(MINUTE);

    let (status, messages) = get(&mut run, json!({"id": [1]}));

    assert_eq!(status, "Accepted");
    assert_eq!(ids(&messages), vec![1]);
}
