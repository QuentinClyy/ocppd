use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::protocol::v201::event::EventKind;
use serde_json::{Value, json};

fn charging(run: &mut Run, authorized: Value) -> String {
    let ev = EvSpec::default();
    run.input(0, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(0);
    let intent = Intent::Authorize {
        connector: None,
        id_tag: "04A2B3C4".into(),
    };
    run.input(0, Input::Intent(intent));
    run.answer(0, authorized);
    run.answer_all(0);
    let started = run
        .calls
        .iter()
        .find(|(_, action, _)| action == "TransactionEvent");
    let id = &started.unwrap().2["transactionInfo"]["transactionId"];
    id.as_str().unwrap().to_owned()
}

fn accepted() -> Value {
    json!({"idTokenInfo": {"status": "Accepted"}})
}

fn answer_event(run: &mut Run, now: u64, event_type: &str, response: Value) {
    for _ in 0..10 {
        let (_, action, payload) = run.last_call();
        if action == "TransactionEvent" && payload["eventType"] == event_type {
            return run.answer(now, response);
        }
        run.answer(now, json!({}));
    }
    panic!("no {event_type} TransactionEvent");
}

fn costs(run: &Run) -> Vec<(f64, bool)> {
    let costs = run.events.iter().filter_map(|event| match event {
        EventKind::CostShown {
            total_cost,
            final_cost,
            ..
        } => Some((*total_cost, *final_cost)),
        _ => None,
    });
    costs.collect()
}

fn driver_messages(run: &Run) -> Vec<String> {
    let texts = run.events.iter().filter_map(|event| match event {
        EventKind::DriverMessage { text, .. } => Some(text.clone()),
        _ => None,
    });
    texts.collect()
}

fn fallback_tariff(run: &mut Run) {
    assert!(
        run.station
            .configure("TariffCostCtrlr.TariffFallbackMessage", "0.35 EUR/kWh")
    );
}

fn ended_offline_with_fallback_cost() -> Run {
    let mut run = Run::accepted(86_400);
    assert!(run.station.configure(
        "TariffCostCtrlr.TotalCostFallbackMessage",
        "See your app for the cost"
    ));
    charging(&mut run, accepted());
    run.input(MINUTE, Input::Disconnected);
    run
}

#[test]
fn given_charging_ev_when_cost_updated_for_its_transaction_then_answered_and_running_cost_shown_per_i02_fr_02_and_fr_03_and_tc_i_01()
 {
    let mut run = Run::accepted(86_400);
    let transaction_id = charging(&mut run, accepted());

    let response = run.result(
        0,
        "CostUpdated",
        json!({"totalCost": 1.5, "transactionId": transaction_id}),
    );

    assert_eq!(response, json!({}));
    assert_eq!(costs(&run), vec![(1.5, false)]);
}

#[test]
fn given_charging_ev_when_cost_updated_for_another_transaction_then_answered_without_cost_shown() {
    let mut run = Run::accepted(86_400);
    charging(&mut run, accepted());
    let unknown = json!({"totalCost": 9.0, "transactionId": "elsewhere"});

    let response = run.result(0, "CostUpdated", unknown);

    assert_eq!(response, json!({}));
    assert!(costs(&run).is_empty());
}

#[test]
fn given_charging_ev_when_updated_event_answered_with_a_total_cost_then_running_cost_shown_per_i02_fr_04_and_tc_i_07()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, accepted());
    run.input(MINUTE, Input::Tick);

    answer_event(&mut run, MINUTE, "Updated", json!({"totalCost": 2.4}));

    assert_eq!(costs(&run), vec![(2.4, false)]);
}

#[test]
fn given_ended_transaction_when_ended_event_answered_with_a_total_cost_then_final_cost_shown_per_i03_fr_03_and_tc_i_02()
 {
    let mut run = Run::accepted(86_400);
    charging(&mut run, accepted());
    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));

    answer_event(&mut run, MINUTE, "Ended", json!({"totalCost": 7.25}));

    assert_eq!(costs(&run), vec![(7.25, true)]);
}

#[test]
fn given_offline_station_with_cost_fallback_when_transaction_ends_then_fallback_message_shown_per_i05_fr_02()
 {
    let mut run = ended_offline_with_fallback_cost();

    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));

    assert_eq!(driver_messages(&run), vec!["See your app for the cost"]);
}

#[test]
fn given_transaction_ended_offline_when_its_cost_arrives_later_then_not_shown_per_i03_fr_03() {
    let mut run = ended_offline_with_fallback_cost();
    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));
    run.input(2 * MINUTE, Input::Connected);

    answer_event(&mut run, 2 * MINUTE, "Ended", json!({"totalCost": 7.25}));

    assert_eq!(costs(&run), vec![], "the driver has left");
}

#[test]
fn given_plugged_ev_when_authorized_with_a_personal_tariff_then_it_is_shown_per_i01_fr_03() {
    let mut run = Run::accepted(86_400);
    let personal = json!({"idTokenInfo": {"status": "Accepted", "personalMessage": {"format": "UTF8", "content": "0.25 EUR/kWh for you"}}});

    charging(&mut run, personal);

    assert_eq!(driver_messages(&run), vec!["0.25 EUR/kWh for you"]);
}

#[test]
fn given_no_tariff_fallback_when_authorized_without_personal_tariff_then_nothing_shown_per_i04_fr_01()
 {
    let mut run = Run::accepted(86_400);

    charging(&mut run, accepted());

    assert!(driver_messages(&run).is_empty());
}

#[test]
fn given_tariff_fallback_when_authorized_without_personal_tariff_then_fallback_shown_per_i04_fr_01()
{
    let mut run = Run::accepted(86_400);
    fallback_tariff(&mut run);

    charging(&mut run, accepted());

    assert_eq!(driver_messages(&run), vec!["0.35 EUR/kWh"]);
}

#[test]
fn given_tariff_fallback_and_cached_id_token_when_authorized_locally_then_fallback_shown_again_per_i04_fr_01()
 {
    let mut run = Run::accepted(86_400);
    fallback_tariff(&mut run);
    charging(&mut run, accepted());
    run.input(MINUTE, Input::Intent(Intent::Unplug { connector: 1 }));
    run.answer_all(MINUTE);
    let ev = EvSpec::default();
    run.input(MINUTE, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    run.answer_all(MINUTE);
    let intent = Intent::Authorize {
        connector: None,
        id_tag: "04A2B3C4".into(),
    };

    run.input(MINUTE, Input::Intent(intent));
    run.answer_all(MINUTE);

    assert_eq!(
        driver_messages(&run),
        vec!["0.35 EUR/kWh", "0.35 EUR/kWh"],
        "the second time from the cache"
    );
}

#[test]
fn given_charging_ev_when_updated_event_answered_with_a_new_tariff_then_it_is_shown_per_i06_fr_03()
{
    let mut run = Run::accepted(86_400);
    charging(&mut run, accepted());
    run.input(MINUTE, Input::Tick);
    let updated =
        json!({"updatedPersonalMessage": {"format": "UTF8", "content": "Now 0.32 EUR/kWh"}});

    answer_event(&mut run, MINUTE, "Updated", updated);

    assert_eq!(driver_messages(&run), vec!["Now 0.32 EUR/kWh"]);
}
