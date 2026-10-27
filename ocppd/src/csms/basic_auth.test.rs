use super::*;

#[test]
fn given_rfc_4648_vectors_when_encoded_then_their_base64() {
    let vectors = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];

    for (plain, encoded) in vectors {
        let found = base64(plain.as_bytes());

        assert_eq!(found, encoded);
    }
}

#[test]
fn given_user_and_password_when_header_built_then_basic_with_both_joined_by_a_colon() {
    let password = b"secret";

    let header = header("CP1", password);

    assert_eq!(header, "Basic Q1AxOnNlY3JldA==");
}
