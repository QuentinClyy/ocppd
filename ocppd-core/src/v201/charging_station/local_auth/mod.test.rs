use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::api::io::Input;
use crate::fixtures::v201::local_auth::*;
use crate::fixtures::v201::station::{MINUTE, Run};
use crate::v201::event::EventKind;
use serde_json::json;

#[test]
fn given_offline_station_when_unknown_id_token_presented_then_refused_per_c15_fr_08() {
    let mut run = Run::accepted(86_400);
    run.input(1_000, Input::Disconnected);

    swipe(&mut run, 2_000, CARD);

    assert!(matches!(
        run.events.last(),
        Some(EventKind::Link(LinkEvent::IntentRejected { .. }))
    ));
}

#[test]
fn given_offline_unknown_id_tokens_allowed_when_back_online_after_a_session_then_authorized_offline_per_c15_fr_01_and_tc_c_26()
 {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "AuthCtrlr.OfflineTxForUnknownIdEnabled", "true");
    run.input(1_000, Input::Disconnected);
    let ev = EvSpec::default();
    run.input(2_000, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    swipe(&mut run, 2_000, CARD);
    run.input(3_000, Input::Intent(Intent::Unplug { connector: 1 }));

    run.input(4_000, Input::Connected);
    run.answer_all(4_000);

    assert_eq!(events(&run)[1]["triggerReason"], "Authorized");
}

#[test]
fn given_unknown_id_token_used_offline_when_it_starts_online_then_csms_asked_per_c15_fr_01() {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "AuthCtrlr.OfflineTxForUnknownIdEnabled", "true");
    run.input(1_000, Input::Disconnected);
    let ev = EvSpec::default();
    run.input(2_000, Input::Intent(Intent::PlugIn { connector: 1, ev }));
    swipe(&mut run, 2_000, CARD);
    run.input(3_000, Input::Intent(Intent::Unplug { connector: 1 }));
    run.input(4_000, Input::Connected);
    run.answer_all(4_000);

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 1, "never cached");
}

#[test]
fn given_listed_master_pass_and_charging_ev_when_master_pass_presented_then_stopped_without_authorize_per_c16_fr_05()
 {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "AuthCtrlr.MasterPassGroupId", "POLICE");
    let officer = json!({"idToken": {"idToken": "04F0F1F2", "type": "ISO14443"}, "idTokenInfo": {"status": "Accepted", "groupIdToken": {"idToken": "POLICE", "type": "Central"}}});
    send(
        &mut run,
        list(json!([entry(CARD, "Accepted"), officer]), 1, "Full"),
    );
    plug_in(&mut run, 0);
    swipe(&mut run, 0, CARD);
    run.answer_all(0);

    swipe(&mut run, MINUTE, "04F0F1F2");
    run.answer_all(MINUTE);

    let ended = events(&run).pop().unwrap();
    assert_eq!(ended["transactionInfo"]["stoppedReason"], "MasterPass");
    assert_eq!(authorizes(&run), 0);
}
