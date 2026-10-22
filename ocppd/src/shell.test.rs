use super::*;

#[test]
fn given_repeated_failures_without_jitter_when_backing_off_then_doubles_from_5_seconds_up_to_5_minutes()
 {
    let failures = 1..=9;

    let delays: Vec<u64> = failures
        .map(|failures| backoff("CP1", failures, false).as_secs())
        .collect();

    assert_eq!(delays, vec![5, 10, 20, 40, 80, 160, 300, 300, 300]);
}

#[test]
fn given_jitter_when_backing_off_then_within_half_and_one_and_a_half_of_the_plain_delay() {
    for failures in 1..=9 {
        let plain = backoff("CP1", failures, false).as_secs_f64();

        let spread = backoff("CP1", failures, true).as_secs_f64();

        assert!(
            (plain * 0.5..plain * 1.5).contains(&spread),
            "{failures}: {spread}"
        );
    }
}

#[test]
fn given_tls_verify_failure_when_classified_then_csms_certificate() {
    let refused = "TLS error: native-tls error: error:0A000086:SSL routines:tls_post_process_server_certificate:certificate verify failed:../ssl/statem/statem_clnt.c:2124: (unable to get local issuer certificate)";

    let kind = failure(refused);

    assert_eq!(kind, ConnectFailure::CsmsCertificate);
}

#[test]
fn given_refused_tcp_connection_when_classified_then_other() {
    let closed = "IO error: Connection refused (os error 111)";

    let kind = failure(closed);

    assert_eq!(kind, ConnectFailure::Other);
}
