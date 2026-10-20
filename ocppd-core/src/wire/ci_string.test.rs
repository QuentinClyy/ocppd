use super::*;

#[test]
fn given_text_at_the_limit_when_wrapped_then_accepted() {
    let text = "abc";

    let wrapped = CiString::<3>::new(text);

    assert!(wrapped.is_ok());
}

#[test]
fn given_text_over_the_limit_when_wrapped_then_refused_with_its_length() {
    let text = "abcd";

    let error = CiString::<3>::new(text).unwrap_err();

    assert_eq!(error.to_string(), "\"abcd\" is longer than 3 characters");
}

#[test]
fn given_json_strings_when_decoded_then_the_limit_checked() {
    let texts = ["\"abc\"", "\"abcd\""];

    let decoded = texts.map(|text| serde_json::from_str::<CiString<3>>(text).is_ok());

    assert_eq!(decoded, [true, false]);
}
