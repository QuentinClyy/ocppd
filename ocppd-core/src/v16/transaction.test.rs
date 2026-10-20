use crate::fixtures::v16::fake_csms::{Csms, authorize};
use crate::intent::{EvSpec, Intent};
use crate::v16::messages::status::ChargePointStatus;
use serde_json::json;

fn plug_in() -> Intent {
    Intent::PlugIn {
        connector: 1,
        ev: EvSpec::default(),
    }
}

fn swiped() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(0, plug_in());
    csms.intent(0, authorize("TAG1"));
    csms
}

#[test]
fn given_start_transaction_sent_when_answered_invalid_then_stopped_as_deauthorized() {
    let mut csms = swiped();
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));

    csms.answer(
        0,
        json!({"idTagInfo": {"status": "Invalid"}, "transactionId": 7}),
    );

    let stop = &csms.sent("StopTransaction")[0];
    assert_eq!(stop["reason"], "DeAuthorized");
    assert_eq!(stop["meterStop"], 0);
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Finishing));
}
