use super::*;

fn args(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

fn run(csms: &str) -> Result<Cli, String> {
    Ok(Cli::Run {
        csms: csms.into(),
        json: false,
        listen: Vec::new(),
    })
}

#[test]
fn given_csms_flag_when_parsed_then_run_against_it_without_trailing_slash() {
    let given = args("--csms ws://csms.example/ocpp/");

    let cli = parse(&given, None);

    assert_eq!(cli, run("ws://csms.example/ocpp"));
}

#[test]
fn given_csms_flag_with_equals_when_parsed_then_run_against_it() {
    let given = args("--csms=ws://csms.example");

    let cli = parse(&given, None);

    assert_eq!(cli, run("ws://csms.example"));
}

#[test]
fn given_no_flag_and_ocppd_csms_set_when_parsed_then_run_against_the_env_url() {
    let given = args("");

    let cli = parse(&given, Some("ws://env.example".into()));

    assert_eq!(cli, run("ws://env.example"));
}

#[test]
fn given_flag_and_ocppd_csms_set_when_parsed_then_the_flag_wins() {
    let given = args("--csms ws://flag.example");

    let cli = parse(&given, Some("ws://env.example".into()));

    assert_eq!(cli, run("ws://flag.example"));
}

#[test]
fn given_help_or_version_flags_when_parsed_then_help_or_version_even_without_csms() {
    let cases = [
        ("-h", Cli::Help),
        ("--help", Cli::Help),
        ("-V", Cli::Version),
        ("--version", Cli::Version),
    ];

    for (flag, expected) in cases {
        let cli = parse(&args(flag), None);

        assert_eq!(cli, Ok(expected), "{flag}");
    }
}

#[test]
fn given_no_csms_at_all_when_parsed_then_usage_error() {
    let given = args("");

    let cli = parse(&given, Some(String::new()));

    assert_eq!(cli, Err("missing --csms <url> (or OCPPD_CSMS)".into()));
}

#[test]
fn given_csms_flag_without_url_when_parsed_then_usage_error() {
    let given = args("--csms");

    let cli = parse(&given, None);

    assert_eq!(cli, Err("--csms needs a url".into()));
}

#[test]
fn given_unknown_argument_when_parsed_then_usage_error_naming_it() {
    let given = args("--csms ws://csms.example --verbose");

    let cli = parse(&given, None);

    assert_eq!(cli, Err("unknown argument '--verbose'".into()));
}

#[test]
fn given_json_flag_when_parsed_then_run_in_json_lines_mode() {
    let given = args("--json --csms ws://csms.example");

    let cli = parse(&given, None);

    let expected = Cli::Run {
        csms: "ws://csms.example".into(),
        json: true,
        listen: Vec::new(),
    };
    assert_eq!(cli, Ok(expected));
}

#[test]
fn given_two_listen_flags_when_parsed_then_both_addresses_in_order() {
    let given =
        args("--csms ws://csms.example --listen unix:/tmp/ocppd.sock --listen=tcp:127.0.0.1:7600");

    let cli = parse(&given, None);

    let expected = Cli::Run {
        csms: "ws://csms.example".into(),
        json: false,
        listen: vec![
            Address::Unix("/tmp/ocppd.sock".into()),
            Address::Tcp("127.0.0.1:7600".into()),
        ],
    };
    assert_eq!(cli, Ok(expected));
}

#[test]
fn given_a_listen_address_without_a_scheme_when_parsed_then_usage_error() {
    let given = args("--csms ws://csms.example --listen /tmp/ocppd.sock");

    let cli = parse(&given, None);

    assert_eq!(
        cli,
        Err("listen address '/tmp/ocppd.sock' must be unix:<path> or tcp:<host>:<port>".into())
    );
}
