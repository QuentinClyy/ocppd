use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize, register};
use crate::protocol::v16::event::EventKind;
use crate::protocol::v16::messages::availability::ChargePointStatus;
use serde_json::json;

fn plugged() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(
        1_000,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
    csms
}

fn swiped() -> Csms {
    let mut csms = plugged();
    csms.intent(2_000, authorize("TAG1"));
    csms
}

fn authorized() -> Csms {
    let mut csms = swiped();
    csms.answer(2_000, json!({"idTagInfo": {"status": "Accepted"}}));
    csms
}

fn started() -> Csms {
    let mut csms = authorized();
    csms.answer(
        2_000,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 42}),
    );
    csms
}

fn stopped() -> Csms {
    let mut csms = started();
    csms.intent(2_000 + 10 * MINUTE, Intent::Stop { connector: 1 });
    csms
}

#[test]
fn given_plugged_ev_when_tag_swiped_then_authorize_sent() {
    let mut csms = plugged();

    csms.intent(2_000, authorize("TAG1"));

    assert_eq!(csms.expect("Authorize"), json!({"idTag": "TAG1"}));
}

#[test]
fn given_swiped_tag_when_authorize_accepted_then_start_transaction_sent() {
    let mut csms = swiped();

    csms.answer(2_000, json!({"idTagInfo": {"status": "Accepted"}}));

    let start = csms.expect("StartTransaction");
    assert_eq!(start["connectorId"], 1);
    assert_eq!(start["idTag"], "TAG1");
    assert_eq!(start["meterStart"], 0);
    assert_eq!(start["timestamp"], "1970-01-01T00:00:02Z");
}

#[test]
fn given_start_transaction_sent_when_answered_then_transaction_started() {
    let mut csms = authorized();

    csms.answer(
        2_000,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 42}),
    );

    assert!(csms.events.contains(&EventKind::TransactionStarted {
        connector: 1,
        transaction_id: 42
    }));
}

#[test]
fn given_started_transaction_when_a_minute_passes_then_meter_values_for_it() {
    let mut csms = started();

    csms.input(2_000 + MINUTE, Input::Tick);

    let meter_values = csms.sent("MeterValues");
    assert_eq!(meter_values[0]["transactionId"], 42);
    assert_eq!(register(&meter_values[0]), 183.0);
}

#[test]
fn given_started_transaction_when_stopped_locally_then_stop_transaction_with_the_register() {
    let mut csms = started();

    csms.intent(2_000 + 10 * MINUTE, Intent::Stop { connector: 1 });

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["reason"], "Local");
    assert_eq!(stop["transactionId"], 42);
    assert_eq!(stop["meterStop"], 1833);
}

#[test]
fn given_stopped_transaction_when_unplugged_then_preparing_charging_finishing_available() {
    let mut csms = stopped();

    csms.intent(2_000 + 11 * MINUTE, Intent::Unplug { connector: 1 });

    use ChargePointStatus::*;
    assert_eq!(
        csms.statuses(),
        vec![Preparing, Charging, Finishing, Available]
    );
}
