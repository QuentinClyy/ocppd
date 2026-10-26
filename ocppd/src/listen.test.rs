use super::*;

#[test]
fn given_unix_and_tcp_addresses_when_parsed_then_each_kind() {
    let unix = Address::parse("unix:/run/user/1000/ocppd.sock");
    let tcp = Address::parse("tcp:127.0.0.1:7600");

    assert_eq!(unix, Ok(Address::Unix("/run/user/1000/ocppd.sock".into())));
    assert_eq!(tcp, Ok(Address::Tcp("127.0.0.1:7600".into())));
}

#[test]
fn given_an_address_without_a_scheme_or_target_when_parsed_then_refused() {
    let cases = ["/tmp/ocppd.sock", "unix:", "tcp:", "udp:127.0.0.1:7600"];

    for case in cases {
        let address = Address::parse(case);

        assert!(address.is_err(), "{case}");
    }
}

#[test]
fn given_a_hello_with_the_token_when_checked_then_it_greets() {
    let hello = r#"{"command":"hello","token":"s3cret"}"#;

    let greeted = greets(hello, "s3cret");

    assert!(greeted);
}

#[test]
fn given_a_wrong_token_or_another_command_when_checked_then_it_does_not_greet() {
    let lines = [
        r#"{"command":"hello","token":"guess"}"#,
        r#"{"command":"spawn","token":"s3cret"}"#,
        "s3cret",
    ];

    for line in lines {
        let greeted = greets(line, "s3cret");

        assert!(!greeted, "{line}");
    }
}
