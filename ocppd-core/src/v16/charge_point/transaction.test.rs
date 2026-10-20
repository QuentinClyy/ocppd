use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, authorize};
use crate::v16::messages::core::status::ChargePointStatus;

fn actions(csms: &Csms) -> Vec<String> {
    csms.sent.iter().map(|(action, _)| action.clone()).collect()
}

fn swiped() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(
        0,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
    csms.intent(0, authorize("TAG1"));
    csms
}

#[test]
fn given_authorize_in_flight_when_disconnected_and_reconnected_then_dropped_and_still_preparing() {
    let mut csms = swiped();
    csms.disconnect(1_000);
    csms.sent.clear();

    csms.reconnect(10_000);

    assert!(!actions(&csms).contains(&"Authorize".to_string()));
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Preparing));
}
