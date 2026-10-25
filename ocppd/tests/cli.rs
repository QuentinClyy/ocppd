use serde_json::Value;
use std::process::{Command, Output};

fn ocppd(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ocppd"))
        .args(args)
        .env_remove("OCPPD_CSMS")
        .output()
        .unwrap()
}

fn description() -> Value {
    let text = include_str!("../../docs/ocppd.opencli.json");
    serde_json::from_str(text).unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn flags(description: &Value) -> Vec<String> {
    let options = description["command"]["options"].as_array().unwrap();
    let names = options.iter().flat_map(|option| {
        let aliases = option["aliases"].as_array().cloned().unwrap_or_default();
        let aliases = aliases
            .into_iter()
            .map(|alias| alias.as_str().unwrap().to_owned());
        std::iter::once(option["name"].as_str().unwrap().to_owned()).chain(aliases)
    });
    names.collect()
}

#[test]
fn given_opencli_description_when_help_printed_then_every_described_flag_is_listed() {
    let flags = flags(&description());

    let help = stdout(&ocppd(&["--help"]));

    for flag in flags {
        assert!(help.contains(&flag), "{flag} missing from --help");
    }
}

#[test]
fn given_help_when_its_flags_read_then_each_is_in_the_opencli_description() {
    let help = stdout(&ocppd(&["--help"]));

    let listed = help
        .split_whitespace()
        .map(|word| word.trim_end_matches(','))
        .filter(|word| word.starts_with('-') && word.len() > 1);

    let described = flags(&description());
    for flag in listed {
        assert!(
            described.iter().any(|known| known == flag),
            "{flag} not described"
        );
    }
}

#[test]
fn given_opencli_description_when_version_printed_then_the_versions_match() {
    let version = description()["info"]["version"]
        .as_str()
        .unwrap()
        .to_owned();

    let printed = stdout(&ocppd(&["-V"]));

    assert_eq!(printed, format!("ocppd {version}\n"));
}

#[test]
fn given_help_flag_when_run_then_help_on_stdout_and_exit_0() {
    let output = ocppd(&["-h"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).starts_with("ocppd - "));
    assert!(output.stderr.is_empty());
}

#[test]
fn given_no_csms_when_run_then_usage_on_stderr_and_the_described_usage_exit_code() {
    let usage_code = description()["command"]["exitCodes"][2]["code"].clone();

    let output = ocppd(&[]);

    assert_eq!(Value::from(output.status.code().unwrap()), usage_code);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("usage: ocppd --csms <url>"), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn given_unknown_argument_when_run_then_usage_error_naming_it() {
    let output = ocppd(&["--csms", "ws://localhost:1", "--verbose"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unknown argument '--verbose'"), "{stderr}");
}
