use crate::api::io::{ConnectFailure, Input};
use crate::fixtures::v201::certificates::{CSMS_ROOT, entries, install, signed_by};
use crate::fixtures::v201::station::Run;
use crate::fixtures::x509::authority::Authority;

fn checked_store() -> (Run, Authority) {
    let mut run = Run::accepted(86_400);
    assert!(
        run.station
            .configure("SecurityCtrlr.AdditionalRootCertificateCheck", "true")
    );
    let old = Authority::new("old root");
    install(&mut run, CSMS_ROOT, &old.pem());
    (run, old)
}

fn replaced_root() -> (Run, String, String) {
    let (mut run, old) = checked_store();
    let new = signed_by(&old, "new root");
    install(&mut run, CSMS_ROOT, &new);
    (run, old.pem(), new)
}

fn refused(run: &mut Run, now: u64) {
    run.input(
        now,
        Input::ConnectFailed(ConnectFailure::CsmsCertificate, String::new()),
    );
}

#[test]
fn given_root_check_when_root_not_signed_by_the_old_one_installed_then_rejected_per_m05_fr_11_and_tc_m_09()
 {
    let (mut run, _) = checked_store();

    let status = install(&mut run, CSMS_ROOT, &Authority::new("stranger").pem());

    assert_eq!(status, "Rejected");
    assert_eq!(entries(&mut run), "1");
}

#[test]
fn given_root_check_when_root_signed_by_the_old_one_installed_then_accepted_and_only_it_trusted_per_m05_fr_12()
 {
    let (mut run, old) = checked_store();
    let new = signed_by(&old, "new root");

    let status = install(&mut run, CSMS_ROOT, &new);

    assert_eq!(status, "Accepted");
    assert_eq!(run.station.csms_roots(), vec![new.into_bytes()]);
    assert_eq!(entries(&mut run), "2", "the old root is kept as a fallback");
}

#[test]
fn given_replaced_root_when_another_root_signed_by_the_old_one_installed_then_rejected_per_m05_fr_10()
 {
    let (mut run, old) = checked_store();
    install(&mut run, CSMS_ROOT, &signed_by(&old, "new root"));

    let status = install(&mut run, CSMS_ROOT, &signed_by(&old, "newer root"));

    assert_eq!(status, "Rejected", "the old root no longer signs");
}

#[test]
fn given_replaced_root_when_connected_with_the_new_one_then_old_root_dropped_per_m05_fr_13_and_tc_m_30()
 {
    let (mut run, _, new) = replaced_root();

    run.reconnect(1_000);

    assert_eq!(run.station.csms_roots(), vec![new.into_bytes()]);
    assert_eq!(entries(&mut run), "1");
}

#[test]
fn given_replaced_root_when_csms_certificate_refused_then_old_root_tried_per_m05_fr_14() {
    let (mut run, old, _) = replaced_root();
    run.input(1_000, Input::Disconnected);

    refused(&mut run, 2_000);

    assert_eq!(run.station.csms_roots(), vec![old.into_bytes()]);
}

#[test]
fn given_old_root_refused_too_when_csms_certificate_refused_then_new_root_tried_again_per_m05_fr_14()
 {
    let (mut run, _, new) = replaced_root();
    run.input(1_000, Input::Disconnected);
    refused(&mut run, 2_000);

    refused(&mut run, 3_000);

    assert_eq!(run.station.csms_roots(), vec![new.into_bytes()]);
}

#[test]
fn given_old_root_tried_when_connected_then_old_root_kept_and_new_dropped_per_m05_fr_14_and_tc_m_31()
 {
    let (mut run, old, _) = replaced_root();
    run.input(1_000, Input::Disconnected);
    refused(&mut run, 2_000);

    run.input(5_000, Input::Connected);

    assert_eq!(run.station.csms_roots(), vec![old.into_bytes()]);
    assert_eq!(entries(&mut run), "1");
}
