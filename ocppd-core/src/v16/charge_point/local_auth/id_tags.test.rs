use crate::api::event::LinkEvent;
use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::{Csms, MINUTE, authorize};
use crate::v16::event::EventKind;
use crate::v16::messages::datatypes::id_tag::AuthorizationStatus;
use serde_json::json;

fn charged_once_then_unplugged() -> Csms {
    let mut csms = Csms::charging(0, EvSpec::default());
    csms.intent(MINUTE, Intent::Unplug { connector: 1 });
    csms
}

fn swipe(csms: &mut Csms, now: u64) -> String {
    let ev = EvSpec::default();
    csms.intent(now, Intent::PlugIn { connector: 1, ev });
    csms.intent(now, authorize("TAG1"));
    let (action, _) = csms.sent.last().expect("nothing sent");
    action.clone()
}

fn pre_authorizing_after_one_charge() -> Csms {
    let mut csms = charged_once_then_unplugged();
    csms.configure(MINUTE, "LocalPreAuthorize", "true");
    csms
}

#[test]
fn given_id_tag_accepted_once_with_local_pre_authorize_when_presented_again_then_started_from_the_cache()
 {
    let mut csms = pre_authorizing_after_one_charge();

    let sent = swipe(&mut csms, 2 * MINUTE);

    assert_eq!(sent, "StartTransaction");
    assert_eq!(csms.sent("Authorize").len(), 1);
}

#[test]
fn given_cached_id_tag_and_local_authorize_offline_when_presented_offline_then_accepted() {
    let mut csms = charged_once_then_unplugged();
    csms.configure(MINUTE, "LocalAuthorizeOffline", "true");
    csms.disconnect(MINUTE);
    csms.events.clear();

    swipe(&mut csms, 2 * MINUTE);

    assert!(csms.events.contains(&EventKind::Authorization {
        connector: Some(1),
        id_tag: "TAG1".into(),
        status: AuthorizationStatus::Accepted,
    }));
    assert!(
        !csms
            .events
            .iter()
            .any(|kind| matches!(kind, EventKind::Link(LinkEvent::IntentRejected { .. })))
    );
}

#[test]
fn given_cleared_cache_when_id_tag_presented_again_then_csms_asked() {
    let mut csms = pre_authorizing_after_one_charge();
    csms.call(MINUTE, "ClearCache", json!({}));

    let sent = swipe(&mut csms, 2 * MINUTE);

    assert_eq!(sent, "Authorize");
}

#[test]
fn given_id_tag_accepted_while_the_cache_was_disabled_when_presented_again_then_csms_asked() {
    let mut csms = Csms::booted();
    csms.configure(0, "AuthorizationCacheEnabled", "false");
    csms.configure(0, "LocalPreAuthorize", "true");
    let ev = EvSpec::default();
    csms.intent(0, Intent::PlugIn { connector: 1, ev });
    csms.intent(0, authorize("TAG1"));
    csms.answer(0, json!({"idTagInfo": {"status": "Accepted"}}));
    csms.answer(
        0,
        json!({"idTagInfo": {"status": "Accepted"}, "transactionId": 1}),
    );
    csms.intent(MINUTE, Intent::Unplug { connector: 1 });
    csms.configure(MINUTE, "AuthorizationCacheEnabled", "true");

    let sent = swipe(&mut csms, 2 * MINUTE);

    assert_eq!(sent, "Authorize");
}

#[test]
fn given_cached_id_tag_listed_as_blocked_when_presented_again_then_csms_asked() {
    let mut csms = pre_authorizing_after_one_charge();
    let list = json!({
        "listVersion": 1,
        "updateType": "Full",
        "localAuthorizationList": [{"idTag": "TAG1", "idTagInfo": {"status": "Blocked"}}],
    });
    csms.call(MINUTE, "SendLocalList", list);

    let sent = swipe(&mut csms, 2 * MINUTE);

    assert_eq!(sent, "Authorize");
}
