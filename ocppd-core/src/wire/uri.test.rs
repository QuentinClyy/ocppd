use super::*;

#[test]
fn given_texts_with_a_scheme_and_no_whitespace_when_parsed_then_uris() {
    for valid in [
        "ftp://x/y.log",
        "https://cs.example/fw",
        "urn:isbn:1",
        "a+b.c-d:x",
    ] {
        let uri = Uri::try_from(valid.to_owned());

        assert!(uri.is_ok(), "{valid}");
    }
}

#[test]
fn given_texts_without_a_valid_scheme_or_with_whitespace_when_parsed_then_refused() {
    for invalid in [
        "",
        "firmware.bin",
        "://x",
        "1ftp://x",
        "ftp://x y",
        "f_p://x",
    ] {
        let uri = Uri::try_from(invalid.to_owned());

        assert!(uri.is_err(), "{invalid}");
    }
}
