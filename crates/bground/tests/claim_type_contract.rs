use bground::ClaimType;
use proptest::prelude::*;
use std::{collections::BTreeSet, str::FromStr};

const EXPECTED_NAMES: [&str; 16] = [
    "file-exists",
    "fn-defined",
    "fn-signature",
    "value-equals",
    "dependency-installed",
    "state-equals",
    "fn-return-type",
    "url-returns",
    "cmd-output-matches",
    "behavior",
    "coherent-refactor",
    "coherent-migration",
    "coherent-spec-impl",
    "coherent-contract",
    "coherent-test-cover",
    "coherent-dep-upgrade",
];

fn claim_type_strategy() -> impl Strategy<Value = ClaimType> {
    prop::sample::select(&ClaimType::ALL)
}

fn assert_rejected(value: &str) {
    assert!(ClaimType::from_str(value).is_err(), "accepted {value:?}");
}

proptest! {
    #[test]
    fn supported_claim_types_round_trip(claim_type in claim_type_strategy()) {
        let stable_name = claim_type.to_string();
        prop_assert_eq!(ClaimType::from_str(&stable_name), Ok(claim_type));
        prop_assert_eq!(ClaimType::from_str(&stable_name)?.stable_name(), stable_name);
    }
}

#[test]
fn supported_claim_type_set_is_closed_ordered_and_unique() {
    let actual_names = ClaimType::ALL.map(|claim_type| claim_type.stable_name());
    let unique_names = actual_names.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(actual_names, EXPECTED_NAMES);
    assert_eq!(unique_names.len(), EXPECTED_NAMES.len());
}

#[test]
fn supported_claim_type_variant_names_are_closed_ordered_unique_and_rust_style() {
    let actual_names = ClaimType::ALL.map(|claim_type| claim_type.variant_name());
    let unique_names = actual_names.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(unique_names.len(), EXPECTED_NAMES.len());

    for name in actual_names {
        assert!(!name.is_empty());
        assert!(name.is_ascii());
        assert!(
            name.bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
        );
        assert!(name.bytes().all(|byte| byte.is_ascii_alphanumeric()));
    }
}

#[test]
fn supported_claim_type_names_use_lowercase_kebab_case() {
    for claim_type in ClaimType::ALL {
        let name = claim_type.stable_name();

        assert!(!name.is_empty());
        assert!(name.is_ascii());
        assert!(!name.starts_with('-'));
        assert!(!name.ends_with('-'));
        assert!(!name.contains("--"));
        assert!(
            name.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        );
    }
}

#[test]
fn unsupported_claim_type_names_are_rejected_without_normalization() {
    for value in [
        "",
        " ",
        "file_exists",
        "FileExists",
        "FILE-EXISTS",
        "file-exists ",
        " file-exists",
        "unknown",
        "coherent-runbook-alignment",
        "coherent-perf-baseline",
    ] {
        assert_rejected(value);
    }
}

#[test]
fn excluded_candidate_names_do_not_enter_supported_set() {
    let supported = ClaimType::ALL
        .iter()
        .map(|claim_type| claim_type.stable_name())
        .collect::<BTreeSet<_>>();

    for excluded in ClaimType::EXCLUDED_CANDIDATES {
        assert!(!supported.contains(excluded));
        assert_rejected(excluded);
    }
}
