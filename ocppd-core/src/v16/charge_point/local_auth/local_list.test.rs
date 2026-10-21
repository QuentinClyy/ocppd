use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, authorize};
use crate::v16::event::EventKind;
use crate::wire::frame::Frame;
use serde_json::{Value, json};

fn result(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { payload, .. } => payload,
        other => panic!("expected a result, got {other:?}"),
    }
}

fn send(csms: &mut Csms, version: i32, update_type: &str, list: Value) -> Value {
    let request = json!({
        "listVersion": version,
        "updateType": update_type,
        "localAuthorizationList": list,
    });
    result(csms.call(0, "SendLocalList", request))["status"].clone()
}

fn version(csms: &mut Csms) -> Value {
    result(csms.call(0, "GetLocalListVersion", json!({})))["listVersion"].clone()
}

fn accepted(id_tag: &str) -> Value {
    json!({"idTag": id_tag, "idTagInfo": {"status": "Accepted"}})
}

fn tags(count: usize) -> Value {
    (0..count).map(|n| accepted(&format!("TAG{n}"))).collect()
}

fn entries(csms: &Csms) -> Option<usize> {
    csms.events.iter().rev().find_map(|kind| match kind {
        EventKind::LocalListUpdated { entries, .. } => Some(*entries),
        _ => None,
    })
}

fn pre_authorizing(list: Value) -> Csms {
    let mut csms = Csms::booted();
    assert_eq!(csms.configure(0, "LocalPreAuthorize", "true"), "Accepted");
    send(&mut csms, 1, "Full", list);
    csms
}

fn plug_in(csms: &mut Csms) {
    let ev = EvSpec::default();
    csms.intent(0, Intent::PlugIn { connector: 1, ev });
}

fn swipe_after_plug_in(csms: &mut Csms) -> String {
    plug_in(csms);
    csms.intent(0, authorize("tag1"));
    let (action, _) = csms.sent.last().expect("nothing sent");
    action.clone()
}

fn listed_v3() -> Csms {
    let mut csms = Csms::booted();
    let list = json!([accepted("TAG1"), accepted("TAG2")]);
    send(&mut csms, 3, "Full", list);
    csms
}

fn listed_v5() -> Csms {
    let mut csms = Csms::booted();
    send(&mut csms, 5, "Full", json!([accepted("TAG1")]));
    csms
}

#[test]
fn given_no_list_when_its_version_asked_then_0() {
    let mut csms = Csms::booted();

    let version = version(&mut csms);

    assert_eq!(version, 0);
}

#[test]
fn given_booted_charge_point_when_full_list_sent_then_accepted_with_its_entries() {
    let mut csms = Csms::booted();
    let list = json!([accepted("TAG1"), accepted("TAG2")]);

    let status = send(&mut csms, 3, "Full", list);

    assert_eq!(status, "Accepted");
    assert_eq!(entries(&csms), Some(2));
}

#[test]
fn given_full_list_version_3_when_its_version_asked_then_3() {
    let mut csms = listed_v3();

    let version = version(&mut csms);

    assert_eq!(version, 3);
}

#[test]
fn given_list_when_empty_full_list_sent_then_accepted_and_empty() {
    let mut csms = listed_v3();

    let status = send(&mut csms, 4, "Full", json!([]));

    assert_eq!(status, "Accepted");
    assert_eq!(entries(&csms), Some(0));
}

#[test]
fn given_list_of_two_when_differential_changes_adds_and_removes_then_accepted_with_the_new_version_and_entries()
 {
    let mut csms = Csms::booted();
    send(
        &mut csms,
        1,
        "Full",
        json!([accepted("TAG1"), accepted("TAG2")]),
    );
    let changes = json!([
        {"idTag": "tag1", "idTagInfo": {"status": "Blocked"}},
        {"idTag": "TAG2"},
        accepted("TAG3"),
    ]);

    let status = send(&mut csms, 2, "Differential", changes);

    assert_eq!(status, "Accepted");
    assert_eq!(version(&mut csms), 2);
    assert_eq!(entries(&csms), Some(2));
}

#[test]
fn given_list_version_5_when_differential_versions_4_and_5_sent_then_version_mismatch_and_list_kept_per_7_47()
 {
    let mut csms = listed_v5();

    let statuses: Vec<_> = [4, 5]
        .iter()
        .map(|stale| send(&mut csms, *stale, "Differential", json!([accepted("TAG2")])))
        .collect();

    assert_eq!(statuses, vec![json!("VersionMismatch"); 2]);
    assert_eq!(version(&mut csms), json!(5));
    assert_eq!(entries(&csms), Some(1));
}

#[test]
fn given_booted_charge_point_when_list_versions_0_and_minus_1_sent_then_failed_per_erratum_3_52() {
    let mut csms = Csms::booted();

    let statuses: Vec<_> = [0, -1]
        .iter()
        .map(|reserved| send(&mut csms, *reserved, "Full", json!([])))
        .collect();

    assert_eq!(statuses, vec![json!("Failed"); 2]);
    assert_eq!(version(&mut csms), json!(0));
}

#[test]
fn given_booted_charge_point_when_101_entries_sent_then_failed() {
    let mut csms = Csms::booted();

    let status = send(&mut csms, 1, "Full", tags(101));

    assert_eq!(status, "Failed");
}

#[test]
fn given_full_list_of_100_when_one_more_added_then_failed_and_list_kept() {
    let mut csms = Csms::booted();
    send(&mut csms, 2, "Full", tags(100));

    let status = send(&mut csms, 3, "Differential", json!([accepted("EXTRA")]));

    assert_eq!(status, "Failed");
    assert_eq!(version(&mut csms), 2);
    assert_eq!(entries(&csms), Some(100));
}

#[test]
fn given_booted_charge_point_when_100_entries_sent_then_accepted() {
    let mut csms = Csms::booted();

    let status = send(&mut csms, 2, "Full", tags(100));

    assert_eq!(status, "Accepted");
}

#[test]
fn given_listed_id_tag_and_local_pre_authorize_when_presented_after_plug_in_then_started_without_authorize()
 {
    let mut csms = pre_authorizing(json!([accepted("TAG1")]));

    let sent = swipe_after_plug_in(&mut csms);

    assert_eq!(sent, "StartTransaction");
    assert!(csms.sent("Authorize").is_empty());
}

#[test]
fn given_listed_id_tag_presented_before_plug_in_when_ev_plugged_then_started_without_authorize() {
    let mut csms = pre_authorizing(json!([accepted("TAG1")]));
    csms.intent(0, authorize("TAG1"));

    plug_in(&mut csms);

    assert_eq!(csms.expect("StartTransaction")["idTag"], "TAG1");
    assert!(csms.sent("Authorize").is_empty());
}

#[test]
fn given_listed_id_tag_without_local_pre_authorize_when_presented_then_csms_asked() {
    let mut csms = Csms::booted();
    send(&mut csms, 1, "Full", json!([accepted("TAG1")]));

    let sent = swipe_after_plug_in(&mut csms);

    assert_eq!(sent, "Authorize");
}

#[test]
fn given_listed_id_tag_and_disabled_list_when_presented_then_csms_asked() {
    let mut csms = pre_authorizing(json!([accepted("TAG1")]));
    assert_eq!(
        csms.configure(0, "LocalAuthListEnabled", "false"),
        "Accepted"
    );

    let sent = swipe_after_plug_in(&mut csms);

    assert_eq!(sent, "Authorize");
}

#[test]
fn given_id_tag_listed_as_blocked_when_presented_then_csms_asked() {
    let blocked = json!([{"idTag": "TAG1", "idTagInfo": {"status": "Blocked"}}]);
    let mut csms = pre_authorizing(blocked);

    let sent = swipe_after_plug_in(&mut csms);

    assert_eq!(sent, "Authorize");
}

#[test]
fn given_id_tag_listed_with_a_past_expiry_when_presented_then_csms_asked() {
    let expired = json!([{"idTag": "TAG1", "idTagInfo": {
        "status": "Accepted",
        "expiryDate": "1970-01-01T00:00:00Z",
    }}]);
    let mut csms = pre_authorizing(expired);

    let sent = swipe_after_plug_in(&mut csms);

    assert_eq!(sent, "Authorize");
}

#[test]
fn given_booted_charge_point_when_boolean_key_set_to_yes_then_rejected() {
    let mut csms = Csms::booted();

    let status = csms.configure(0, "LocalPreAuthorize", "yes");

    assert_eq!(status, "Rejected");
}
