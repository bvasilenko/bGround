mod common;

use assert_cmd::Command;
use bground::{ClaimString, ClaimType, EvidenceState};
use bsuite_core::ExitCode;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct ClaimCase {
    label: &'static str,
    claim: &'static str,
}

fn bground_command() -> Command {
    Command::cargo_bin("bground").unwrap()
}

fn usage_code() -> i32 {
    ExitCode::Usage.as_i32()
}

fn finding_code() -> i32 {
    ExitCode::Finding.as_i32()
}

fn assert_placeholder_directive_bytes(
    output: &[u8],
    claim: &bground::ParsedClaim,
    evidence_state: EvidenceState,
) {
    let output = String::from_utf8(output.to_vec()).unwrap();

    common::assert_placeholder_directive(&output, claim, evidence_state);
}

fn assert_unique_claim_cases(cases: &[ClaimCase]) {
    let labels = cases.iter().map(|case| case.label).collect::<BTreeSet<_>>();
    let claims = cases.iter().map(|case| case.claim).collect::<BTreeSet<_>>();

    assert_eq!(labels.len(), cases.len(), "duplicate claim-case label");
    assert_eq!(claims.len(), cases.len(), "duplicate claim-case input");
}

#[test]
fn help_exits_successfully_and_describes_public_commands() {
    let output = bground_command()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();

    for token in ["verify", "claim-types", "update", "init", "tail", "explain"] {
        assert!(output.contains(token), "help omitted {token}");
    }
}

#[test]
fn claim_types_prints_supported_names_only_once_each() {
    let output = bground_command()
        .arg("claim-types")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let lines = output.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), ClaimType::ALL.len());

    for claim_type in ClaimType::ALL {
        let occurrences = lines
            .iter()
            .filter(|line| **line == claim_type.stable_name())
            .count();

        assert_eq!(occurrences, 1, "unexpected count for {claim_type}");
    }

    for excluded in ClaimType::EXCLUDED_CANDIDATES {
        assert!(!lines.contains(&excluded));
    }
}

#[test]
fn verify_accepts_valid_evidence_shape_with_equals_inside_value() {
    let claim = "file-exists:README.md:README exists";
    let output = bground_command()
        .args(["verify", claim, "--evidence", "id=value=kept"])
        .assert()
        .code(finding_code())
        .get_output()
        .stdout
        .clone();

    let parsed = ClaimString::parse(claim).unwrap();

    assert_placeholder_directive_bytes(&output, &parsed, EvidenceState::Ungrounded);
}

#[test]
fn verify_accepts_every_supported_claim_type_before_placeholder_verdict() {
    for claim_type in ClaimType::ALL {
        let claim = format!("{claim_type}:target:assertion");
        let parsed = ClaimString::parse(&claim).unwrap();
        let output = bground_command()
            .args(["verify", &claim])
            .assert()
            .code(finding_code())
            .get_output()
            .stdout
            .clone();

        assert_placeholder_directive_bytes(&output, &parsed, EvidenceState::Ungrounded);
    }
}

#[test]
fn verify_preserves_assertion_delimiters_before_placeholder_verdict() {
    let claim = "value-equals:key:value:with:colon";

    assert!(ClaimString::parse(claim).is_ok());
    let output = bground_command()
        .args(["verify", claim])
        .assert()
        .code(finding_code())
        .get_output()
        .stdout
        .clone();

    let parsed = ClaimString::parse(claim).unwrap();

    assert_placeholder_directive_bytes(&output, &parsed, EvidenceState::Ungrounded);
}

#[test]
fn verify_valid_optional_flags_do_not_suppress_the_directive() {
    let claim = "behavior:cli:json quiet reason and manifest flags remain pre-corpus";
    let parsed = ClaimString::parse(claim).unwrap();
    let output = bground_command()
        .args([
            "verify",
            claim,
            "--json",
            "--quiet",
            "--reason",
            "operator-requested-check",
            "--manifest",
            "manifest.json",
        ])
        .assert()
        .code(finding_code())
        .get_output()
        .stdout
        .clone();

    assert_placeholder_directive_bytes(&output, &parsed, EvidenceState::Ungrounded);
}

#[test]
fn verify_rejects_invalid_evidence_shape_before_dispatch() {
    bground_command()
        .args([
            "verify",
            "file-exists:README.md:README exists",
            "--evidence",
            "missing-separator",
        ])
        .assert()
        .failure();
}

#[test]
fn verify_rejects_unsupported_claim_type_families_before_placeholder_verdict() {
    let mut unsupported_names = vec!["unknown"];
    unsupported_names.extend(ClaimType::EXCLUDED_CANDIDATES);

    for unsupported_name in unsupported_names {
        let claim = format!("{unsupported_name}:target:assertion");

        assert!(ClaimString::parse(&claim).is_err());
        bground_command()
            .args(["verify", &claim])
            .assert()
            .code(usage_code());
    }
}

#[test]
fn verify_rejects_malformed_claim_strings_before_placeholder_verdict() {
    let cases = [
        ClaimCase {
            label: "empty",
            claim: "",
        },
        ClaimCase {
            label: "missing-target-and-assertion",
            claim: "file-exists",
        },
        ClaimCase {
            label: "missing-assertion",
            claim: "file-exists:target",
        },
        ClaimCase {
            label: "empty-type",
            claim: ":target:assertion",
        },
        ClaimCase {
            label: "empty-target",
            claim: "file-exists::assertion",
        },
        ClaimCase {
            label: "empty-assertion",
            claim: "file-exists:target:",
        },
    ];

    assert_unique_claim_cases(&cases);

    for case in cases {
        assert!(ClaimString::parse(case.claim).is_err(), "{}", case.label);
        bground_command()
            .args(["verify", case.claim])
            .assert()
            .code(usage_code());
    }
}
