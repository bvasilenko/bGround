use bground::{BgroundError, ClaimString, ClaimType, EvidenceMap};

fn assert_claim(value: &str, claim_type: ClaimType, target: &str, assertion: &str) {
    let parsed = ClaimString::parse(value).unwrap();

    assert_eq!(parsed.claim_type, claim_type);
    assert_eq!(parsed.target, target);
    assert_eq!(parsed.assertion, assertion);
    assert_eq!(parsed.to_string(), value);
}

fn assert_malformed(value: &str) {
    assert!(
        matches!(
            ClaimString::parse(value),
            Err(BgroundError::ClaimStringMalformed(_))
        ),
        "accepted malformed claim {value:?}"
    );
}

#[test]
fn parses_claim_strings_without_losing_delimiters_inside_assertions() {
    for (value, claim_type, target, assertion) in [
        (
            "file-exists:README.md:file exists",
            ClaimType::FileExists,
            "README.md",
            "file exists",
        ),
        (
            "behavior:cli:help exits 0",
            ClaimType::Behavior,
            "cli",
            "help exits 0",
        ),
        (
            "value-equals:key:value:with:colon",
            ClaimType::ValueEquals,
            "key",
            "value:with:colon",
        ),
    ] {
        assert_claim(value, claim_type, target, assertion);
    }
}

#[test]
fn rejects_claim_strings_with_missing_required_segments() {
    for value in [
        "",
        "file-exists",
        "file-exists:target",
        ":target:assertion",
        "file-exists::assertion",
        "file-exists:target:",
    ] {
        assert_malformed(value);
    }
}

#[test]
fn rejects_unknown_claim_type_without_hiding_shape_errors() {
    assert!(matches!(
        ClaimString::parse("unknown:target:assertion"),
        Err(BgroundError::UnknownClaimType(_))
    ));
}

#[test]
fn parses_evidence_entries_as_ordered_key_value_map() {
    let evidence = EvidenceMap::parse_entries([
        "id=value".to_owned(),
        "other=value=kept".to_owned(),
        "id=latest".to_owned(),
    ])
    .unwrap();

    assert_eq!(evidence.as_inner().len(), 2);
    assert_eq!(evidence.as_inner().get("id"), Some(&"latest".to_owned()));
    assert_eq!(
        evidence.as_inner().get("other"),
        Some(&"value=kept".to_owned())
    );
}

#[test]
fn rejects_invalid_evidence_entries() {
    for value in ["missing_separator", "=missing_key", " =blank_after_trim"] {
        assert!(
            matches!(
                EvidenceMap::parse_entries([value.to_owned()]),
                Err(BgroundError::EvidenceMapInvalid(_))
            ),
            "accepted invalid evidence entry {value:?}"
        );
    }
}

#[test]
fn accepts_empty_evidence_values() {
    let evidence = EvidenceMap::parse_entries(["id=".to_owned()]).unwrap();

    assert_eq!(evidence.as_inner().get("id"), Some(&String::new()));
}
