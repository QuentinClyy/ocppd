use crate::fixtures::v201::local_auth::*;
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::json;

#[test]
fn given_booted_station_when_first_session_then_id_token_authorized_by_the_csms_per_c12_fr_02() {
    let mut run = Run::accepted(86_400);

    session(&mut run, 0, "Accepted");

    assert_eq!(authorizes(&run), 1);
}

#[test]
fn given_cached_accepted_id_token_when_it_starts_again_then_no_authorize_and_charges_per_c12_fr_02()
{
    let mut run = Run::accepted(86_400);
    session(&mut run, 0, "Accepted");

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 1, "from the cache");
    assert_eq!(events(&run).last().unwrap()["eventType"], "Ended");
    assert!(
        events(&run)
            .iter()
            .filter(|event| event.get("idToken").is_some())
            .count()
            >= 2
    );
}

#[test]
fn given_cached_id_token_without_local_pre_authorize_when_it_starts_again_then_csms_asked() {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "AuthCtrlr.LocalPreAuthorize", "false");
    session(&mut run, 0, "Accepted");

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2);
}

#[test]
fn given_cached_blocked_id_token_when_it_starts_again_then_csms_asked_per_c10_fr_03() {
    let mut run = Run::accepted(86_400);
    session(&mut run, 0, "Blocked");

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2);
}

#[test]
fn given_cache_lifetime_passed_when_id_token_starts_again_then_csms_asked_per_c10_fr_10() {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "AuthCacheCtrlr.LifeTime", "60");
    session(&mut run, 0, "Accepted");

    session(&mut run, 2 * MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2);
}

#[test]
fn given_id_token_with_a_past_cache_expiry_when_it_starts_again_then_csms_asked_per_c10_fr_13() {
    let mut run = Run::accepted(86_400);
    let expired = json!({"status": "Accepted", "cacheExpiryDateTime": "1970-01-01T00:00:00Z"});
    session_answered(&mut run, expired, None);

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2, "an expiry in the past is never cached");
}

#[test]
fn given_id_token_blocked_by_a_transaction_event_response_when_it_starts_again_then_csms_asked_per_c10_fr_05()
 {
    let mut run = Run::accepted(86_400);
    let accepted = json!({"status": "Accepted"});
    let blocked = json!({"status": "Blocked"});
    session_answered(&mut run, accepted, Some(blocked));

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2, "now cached as Blocked");
}

#[test]
fn given_cached_id_token_when_cache_cleared_then_accepted_per_c11_fr_03() {
    let mut run = Run::accepted(86_400);
    session(&mut run, 0, "Accepted");

    let status = clear_cache(&mut run);

    assert_eq!(status, "Accepted");
}

#[test]
fn given_cleared_cache_when_id_token_starts_again_then_csms_asked_per_c11_fr_03() {
    let mut run = Run::accepted(86_400);
    session(&mut run, 0, "Accepted");
    clear_cache(&mut run);

    session(&mut run, MINUTE, "Accepted");

    assert_eq!(authorizes(&run), 2);
}

#[test]
fn given_disabled_cache_when_cache_cleared_then_rejected_per_c11_fr_04_and_tc_c_38() {
    let mut run = Run::accepted(86_400);
    configure(&mut run, "AuthCacheCtrlr.Enabled", "false");

    let status = clear_cache(&mut run);

    assert_eq!(status, "Rejected");
}
