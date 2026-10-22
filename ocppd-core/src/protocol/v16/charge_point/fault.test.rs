use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::fixtures::v16::replies::status;
use crate::protocol::v16::messages::core::status::ChargePointStatus;
use serde_json::json;

fn fault(csms: &mut Csms, now: u64) {
    csms.intent(now, Intent::Fault { connector: 1 });
}

fn faulted_with_attempted_start() -> Csms {
    let mut csms = Csms::booted();
    fault(&mut csms, 0);
    let ev = EvSpec::default();
    csms.intent(MINUTE, Intent::PlugIn { connector: 1, ev });
    csms.intent(MINUTE, authorize("TAG1"));
    csms
}

#[test]
fn given_available_connector_when_faulted_then_faulted_with_internal_error_per_tc_041() {
    let mut csms = Csms::booted();

    fault(&mut csms, 0);

    let notification = csms.sent("StatusNotification").pop().unwrap();
    assert_eq!(notification["status"], "Faulted");
    assert_eq!(notification["errorCode"], "InternalError");
}

#[test]
fn given_faulted_connector_when_ev_plugged_and_authorized_then_nothing_starts_per_tc_041() {
    let csms = faulted_with_attempted_start();

    let starts = csms.sent("StartTransaction");

    assert!(starts.is_empty());
}

#[test]
fn given_faulted_connector_with_plugged_ev_when_fault_cleared_then_preparing_per_tc_041() {
    let mut csms = faulted_with_attempted_start();

    csms.intent(2 * MINUTE, Intent::ClearFault { connector: 1 });

    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Preparing));
}

#[test]
fn given_charging_ev_when_connector_faulted_then_transaction_stopped_and_faulted() {
    let mut csms = Csms::charging(0, EvSpec::default());

    fault(&mut csms, MINUTE);

    assert_eq!(csms.sent("StopTransaction")[0]["reason"], "Other");
    assert_eq!(csms.statuses().last(), Some(&ChargePointStatus::Faulted));
}

#[test]
fn given_faulted_connector_when_reserved_then_faulted_per_tc_048_1() {
    let mut csms = Csms::booted();
    fault(&mut csms, 0);
    let reserve = json!({"connectorId": 1, "expiryDate": "1970-01-01T01:00:00Z", "idTag": "TAG1", "reservationId": 1});

    let reply = csms.call(MINUTE, "ReserveNow", reserve);

    assert_eq!(status(reply), "Faulted");
}

#[test]
fn given_faulted_connector_when_remote_start_then_rejected_per_tc_048_1() {
    let mut csms = Csms::booted();
    fault(&mut csms, 0);
    let start = json!({"connectorId": 1, "idTag": "TAG1"});

    let reply = csms.call(MINUTE, "RemoteStartTransaction", start);

    assert_eq!(status(reply), "Rejected");
}
