mod common;

use assert_cmd::Command;
use bground::ClaimType;
use bsuite_core::ExitCode;

fn bground_command() -> Command {
    Command::cargo_bin("bground").unwrap()
}

#[test]
fn verify_routes_directive_to_stdout_and_nothing_to_stderr_for_all_claim_types() {
    for claim_type in ClaimType::ALL {
        let claim = format!("{claim_type}:target:assertion");
        let output = bground_command()
            .args(["verify", &claim])
            .assert()
            .code(ExitCode::Finding.as_i32())
            .get_output()
            .clone();

        assert!(
            !output.stdout.is_empty(),
            "verify {claim_type}: directive must appear on stdout"
        );
        assert!(
            output.stderr.is_empty(),
            "verify {claim_type}: must not write to stderr on success, got: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn malformed_claim_type_routes_error_to_stderr_and_nothing_to_stdout() {
    for claim in common::malformed_claim_type_claims() {
        let output = bground_command()
            .args(["verify", &claim])
            .assert()
            .code(ExitCode::Usage.as_i32())
            .get_output()
            .clone();

        assert!(
            output.stdout.is_empty(),
            "malformed type {claim:?}: must produce no stdout, got: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            !output.stderr.is_empty(),
            "malformed type {claim:?}: must produce error on stderr"
        );
    }
}

#[test]
fn malformed_claim_shape_routes_error_to_stderr_and_nothing_to_stdout() {
    for &claim in common::MALFORMED_CLAIM_SHAPES {
        let output = bground_command()
            .args(["verify", claim])
            .assert()
            .code(ExitCode::Usage.as_i32())
            .get_output()
            .clone();

        assert!(
            output.stdout.is_empty(),
            "malformed shape {claim:?}: must produce no stdout"
        );
        assert!(
            !output.stderr.is_empty(),
            "malformed shape {claim:?}: must produce error on stderr"
        );
    }
}

#[test]
fn deferred_verbs_produce_no_output_on_either_stream_and_exit_successfully() {
    for subcmd in ["init", "tail", "explain"] {
        let output = bground_command()
            .arg(subcmd)
            .assert()
            .code(ExitCode::Success.as_i32())
            .get_output()
            .clone();

        assert!(
            output.stdout.is_empty(),
            "deferred verb {subcmd}: must not write to stdout, got: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            output.stderr.is_empty(),
            "deferred verb {subcmd}: must not write to stderr, got: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn claim_types_listing_goes_to_stdout_not_stderr_and_is_not_json() {
    let output = bground_command()
        .arg("claim-types")
        .assert()
        .code(ExitCode::Success.as_i32())
        .get_output()
        .clone();

    let stdout = String::from_utf8(output.stdout).expect("claim-types stdout is UTF-8");

    // Content completeness (all names present exactly once) is cli_contract's responsibility.
    assert!(
        !stdout.is_empty(),
        "claim-types must produce output on stdout"
    );
    assert!(
        output.stderr.is_empty(),
        "claim-types must not write to stderr, got: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        serde_json::from_str::<serde_json::Value>(stdout.trim()).is_err(),
        "claim-types output must not be valid JSON"
    );
}
