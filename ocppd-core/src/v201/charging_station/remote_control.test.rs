use crate::fixtures::v201::remote_control::{events, plug_in, start};
use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn stop(run: &mut Run, id: &Value) -> Value {
    let status = run.result(0, "RequestStopTransaction", json!({"transactionId": id}));
    run.answer_all(0);
    status
}

fn transaction_id(run: &Run) -> Value {
    events(run)[0]["transactionInfo"]["transactionId"].clone()
}

fn unlock(run: &mut Run, evse: u32, connector: u32) -> Value {
    let request = json!({"evseId": evse, "connectorId": connector});
    run.result(0, "UnlockConnector", request)["status"].clone()
}

fn authorize_remote_start(run: &mut Run) -> Value {
    assert!(
        run.station
            .configure("AuthCtrlr.AuthorizeRemoteStart", "true")
    );
    let request = json!({"idToken": {"idToken": "04A2B3C4", "type": "ISO14443"}, "remoteStartId": 8, "evseId": 1});
    run.result(0, "RequestStartTransaction", request)["status"].clone()
}

#[test]
fn given_booted_station_when_remote_start_on_evse_1_then_started_with_its_remote_start_id_per_f02_and_f01_fr_04_fr_19_fr_25()
 {
    let mut run = Run::accepted(86_400);

    let response = start(&mut run, Some(1));

    assert_eq!(response, json!({"status": "Accepted"}));
    let started = &events(&run)[0];
    assert_eq!(started["eventType"], "Started");
    assert_eq!(started["triggerReason"], "RemoteStart");
    assert_eq!(started["transactionInfo"]["remoteStartId"], 7);
    assert_eq!(started["idToken"]["idToken"], "APP-USER-1");
}

#[test]
fn given_authorize_remote_start_when_remote_start_then_accepted_and_authorize_sent_per_f01_fr_01() {
    let mut run = Run::accepted(86_400);

    let status = authorize_remote_start(&mut run);

    assert_eq!(status, "Accepted");
    assert_eq!(run.last_call().1, "Authorize");
}

#[test]
fn given_remote_start_being_authorized_when_accepted_then_started_as_remote_start_per_f01_fr_01() {
    let mut run = Run::accepted(86_400);
    authorize_remote_start(&mut run);

    run.answer(0, json!({"idTokenInfo": {"status": "Accepted"}}));
    run.answer_all(0);

    assert_eq!(events(&run)[0]["triggerReason"], "RemoteStart");
}

#[test]
fn given_one_evse_when_remote_start_on_evse_2_then_rejected() {
    let mut run = Run::accepted(86_400);

    let response = start(&mut run, Some(2));

    assert_eq!(response["status"], "Rejected");
}

#[test]
fn given_remote_started_evse_when_remote_start_again_then_rejected_per_f01_fr_24() {
    let mut run = Run::accepted(86_400);
    start(&mut run, Some(1));

    let response = start(&mut run, Some(1));

    assert_eq!(response["status"], "Rejected");
}

#[test]
fn given_inoperative_evse_when_remote_start_then_rejected_per_f01_fr_23() {
    let mut run = Run::accepted(86_400);
    let inoperative = json!({"operationalStatus": "Inoperative", "evse": {"id": 1}});
    run.result(0, "ChangeAvailability", inoperative);
    run.answer_all(0);

    let response = start(&mut run, Some(1));

    assert_eq!(response["status"], "Rejected");
}

#[test]
fn given_remote_started_transaction_when_remote_stop_then_ended_as_remote_per_f03_fr_03_and_fr_07()
{
    let mut run = Run::accepted(86_400);
    start(&mut run, Some(1));
    let id = transaction_id(&run);

    let status = stop(&mut run, &id);

    assert_eq!(status, json!({"status": "Accepted"}));
    let ended = events(&run).pop().unwrap();
    assert_eq!(ended["eventType"], "Ended");
    assert_eq!(ended["triggerReason"], "RemoteStop");
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "Remote");
}

#[test]
fn given_booted_station_when_remote_stop_of_unknown_transaction_then_rejected_per_f03_fr_08_and_tc_f_09()
 {
    let mut run = Run::accepted(86_400);

    let status = stop(&mut run, &json!("nope"));

    assert_eq!(status["status"], "Rejected");
}

#[test]
fn given_ev_connected_stop_point_when_remote_stop_then_only_energy_stops_per_f03_fr_02() {
    let mut run = Run::accepted(86_400);
    assert!(run.station.configure("TxCtrlr.TxStopPoint", "EVConnected"));
    plug_in(&mut run, 0);
    start(&mut run, None);
    let id = transaction_id(&run);

    stop(&mut run, &id);

    let updated = events(&run).pop().unwrap();
    assert_eq!(updated["eventType"], "Updated");
    assert_eq!(updated["triggerReason"], "RemoteStop");
    assert_eq!(updated["transactionInfo"]["chargingState"], "EVConnected");
}

#[test]
fn given_one_connector_per_evse_when_connector_2_unlocked_then_unknown_connector_per_f05_fr_03_and_tc_f_10()
 {
    let mut run = Run::accepted(86_400);

    let status = unlock(&mut run, 1, 2);

    assert_eq!(status, "UnknownConnector");
}

#[test]
fn given_no_cable_when_connector_unlocked_then_unlocked_per_f05_fr_06_and_tc_f_07() {
    let mut run = Run::accepted(86_400);

    let status = unlock(&mut run, 1, 1);

    assert_eq!(status, "Unlocked");
}

#[test]
fn given_authorized_transaction_when_connector_unlocked_then_ongoing_authorized_transaction_per_f05_fr_02_and_tc_f_05()
 {
    let mut run = Run::accepted(86_400);
    start(&mut run, Some(1));

    let status = unlock(&mut run, 1, 1);

    assert_eq!(status, "OngoingAuthorizedTransaction");
}
