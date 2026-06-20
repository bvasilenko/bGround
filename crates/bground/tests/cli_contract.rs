mod common;

use assert_cmd::Command;
use bground::ClaimType;
use bsuite_core::ExitCode;
use std::collections::BTreeSet;

fn bground_command() -> Command {
    Command::cargo_bin("bground").unwrap()
}

fn assert_corpus_directive_output(output: &[u8]) {
    let text = String::from_utf8(output.to_vec()).unwrap();
    common::assert_corpus_directive(&text);
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
fn claim_types_prints_each_supported_name_exactly_once() {
    let output = bground_command()
        .arg("claim-types")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(
        lines.len(),
        ClaimType::ALL.len(),
        "claim-types must print exactly one line per supported claim type"
    );

    for claim_type in ClaimType::ALL {
        let occurrences = lines
            .iter()
            .filter(|line| **line == claim_type.stable_name())
            .count();
        assert_eq!(
            occurrences, 1,
            "claim type '{claim_type}' must appear exactly once"
        );
    }

    for excluded in ClaimType::EXCLUDED_CANDIDATES {
        assert!(
            !lines.contains(&excluded),
            "excluded candidate '{excluded}' must not appear in claim-types output"
        );
    }
}

#[test]
fn verify_accepts_valid_evidence_shape_with_equals_inside_value() {
    let output = bground_command()
        .args([
            "verify",
            "file-exists:README.md:README exists",
            "--evidence",
            "id=value=kept",
        ])
        .assert()
        .code(ExitCode::Finding.as_i32())
        .get_output()
        .stdout
        .clone();

    assert_corpus_directive_output(&output);
}

#[test]
fn verify_accepts_every_supported_claim_type() {
    for claim_type in ClaimType::ALL {
        let claim = format!("{claim_type}:target:assertion");
        let output = bground_command()
            .args(["verify", &claim])
            .assert()
            .code(ExitCode::Finding.as_i32())
            .get_output()
            .stdout
            .clone();

        assert_corpus_directive_output(&output);
    }
}

#[test]
fn verify_preserves_assertion_delimiters_in_colon_heavy_claim() {
    let claim = "value-equals:key:value:with:colon";
    let output = bground_command()
        .args(["verify", claim])
        .assert()
        .code(ExitCode::Finding.as_i32())
        .get_output()
        .stdout
        .clone();

    assert_corpus_directive_output(&output);
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
fn verify_rejects_unsupported_claim_type_families() {
    let claims = common::malformed_claim_type_claims();

    let unique: BTreeSet<_> = claims.iter().collect();
    assert_eq!(
        unique.len(),
        claims.len(),
        "malformed_claim_type_claims() contains duplicate entries"
    );

    for claim in &claims {
        bground_command()
            .args(["verify", claim])
            .assert()
            .code(ExitCode::Usage.as_i32());
    }
}

#[test]
fn verify_rejects_malformed_claim_strings() {
    let claims = common::MALFORMED_CLAIM_SHAPES;

    let unique: BTreeSet<_> = claims.iter().collect();
    assert_eq!(
        unique.len(),
        claims.len(),
        "MALFORMED_CLAIM_SHAPES contains duplicate entries"
    );

    for &claim in claims {
        bground_command()
            .args(["verify", claim])
            .assert()
            .code(ExitCode::Usage.as_i32());
    }
}
