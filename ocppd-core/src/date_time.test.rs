use super::*;

fn at(unix_ms: u64) -> DateTime {
    DateTime { unix_ms }
}

#[test]
fn given_known_instants_when_formatted_then_rfc_3339_utc_with_milliseconds_only_when_set() {
    let instants = [0, 1_792_454_400_000, 1_709_208_000_250];

    let texts = instants.map(|unix_ms| at(unix_ms).to_string());

    assert_eq!(
        texts,
        [
            "1970-01-01T00:00:00Z",
            "2026-10-20T00:00:00Z",
            "2024-02-29T12:00:00.250Z"
        ]
    );
}

#[test]
fn given_formatted_instants_when_parsed_then_the_same_instants() {
    for unix_ms in [0, 951_782_400_000, 1_709_208_000_250, 1_792_468_237_322] {
        let text = at(unix_ms).to_string();

        let parsed = text.parse();

        assert_eq!(parsed, Ok(at(unix_ms)));
    }
}

#[test]
fn given_offsets_and_fractions_when_parsed_then_the_utc_instant_to_the_millisecond() {
    let texts = [
        "2026-10-20T02:00:00+02:00",
        "2026-10-19T19:30:00-04:30",
        "2026-10-20T00:00:00.5Z",
        "2026-10-20T00:00:00.123456Z",
    ];

    let parsed = texts.map(|text| text.parse::<DateTime>());

    assert_eq!(
        parsed,
        [
            Ok(at(1_792_454_400_000)),
            Ok(at(1_792_454_400_000)),
            Ok(at(1_792_454_400_500)),
            Ok(at(1_792_454_400_123))
        ]
    );
}

#[test]
fn given_invalid_texts_when_parsed_then_refused() {
    for text in [
        "",
        "garbage",
        "2026-10-20",
        "2026-10-20T00:00:00",
        "2026-02-30T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-10-20T24:00:00Z",
        "2026-10-20T00:00:00.Z",
        "2026-10-20T00:00:00+0200",
        "1969-12-31T23:59:59Z",
        "2026-10-20T00:00:00Zé",
    ] {
        let parsed = text.parse::<DateTime>();

        assert!(parsed.is_err(), "{text}");
    }
}

#[test]
fn given_json_date_time_when_decoded_and_encoded_then_unchanged() {
    let json = "\"2026-10-20T00:00:00Z\"";

    let value: DateTime = serde_json::from_str(json).unwrap();

    assert_eq!(serde_json::to_string(&value).unwrap(), json);
}
