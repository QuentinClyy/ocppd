use super::*;

#[test]
fn given_4_or_7_byte_hex_uids_when_typed_then_iso14443_per_3_43() {
    let uids = ["04A2B3C4", "04a2b3c4d5e6f7"];

    let types = uids.map(token_type);

    assert_eq!(types, [IdTokenType::Iso14443, IdTokenType::Iso14443]);
}

#[test]
fn given_8_byte_hex_uid_when_typed_then_iso15693_per_3_43() {
    let uid = "E004010203040506";

    let kind = token_type(uid);

    assert_eq!(kind, IdTokenType::Iso15693);
}

#[test]
fn given_other_texts_when_typed_then_local_per_3_43() {
    for local in ["TAG1", "04A2B3C", "04A2B3CG", ""] {
        let kind = token_type(local);

        assert_eq!(kind, IdTokenType::Local, "{local}");
    }
}
