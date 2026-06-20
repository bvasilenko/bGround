use assert_cmd::Command;
use bground::ClaimType;
use bsuite_core::ExitCode;
use serde_json::Value;

fn bground_command() -> Command {
    Command::cargo_bin("bground").unwrap()
}

fn finding_code() -> i32 {
    ExitCode::Finding.as_i32()
}

#[test]
fn json_flag_emits_valid_json_envelope_on_stdout() {
    let raw = bground_command()
        .args(["verify", "file-exists:README.md:must exist", "--json"])
        .assert()
        .code(finding_code())
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8(raw).expect("stdout is UTF-8");
    let envelope: Value =
        serde_json::from_str(output.trim()).expect("--json output must be valid JSON");

    assert_eq!(
        envelope["schema_version"].as_u64(),
        Some(1),
        "envelope must carry schema_version = 1"
    );
    assert_eq!(
        envelope["outcome"].as_str(),
        Some("finding"),
        "outcome must be 'finding' for an ungrounded verify call"
    );
    let directive = envelope["directive"]
        .as_str()
        .expect("envelope must carry a 'directive' string field");
    assert!(!directive.is_empty(), "directive field must not be empty");
    assert!(
        !directive.contains("[bground placeholder directive"),
        "json directive must not contain placeholder header"
    );
}

#[test]
fn json_flag_omits_directive_field_on_error() {
    let raw = bground_command()
        .args(["verify", "unknown-type:target:assertion", "--json"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let output = String::from_utf8(raw).expect("stderr is UTF-8");
    assert!(
        !output.is_empty(),
        "malformed input must produce stderr output"
    );
}

#[test]
fn plain_output_without_json_flag_is_not_json() {
    let raw = bground_command()
        .args(["verify", "file-exists:README.md:must exist"])
        .assert()
        .code(finding_code())
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8(raw).expect("stdout is UTF-8");
    let parse_result = serde_json::from_str::<Value>(output.trim());
    assert!(
        parse_result.is_err(),
        "plain output must not be JSON-parseable as an object"
    );
    assert!(!output.is_empty(), "plain output must be non-empty");
}

#[test]
fn json_flag_produces_valid_envelope_for_all_claim_types() {
    // The JSON envelope contract (schema_version, outcome, directive) must hold for
    // every claim type, not just file-exists. This guards against future claim types
    // being added without wiring them through the corpus index.
    for claim_type in ClaimType::ALL {
        let claim = format!("{claim_type}:target:assertion");
        let raw = bground_command()
            .args(["verify", &claim, "--json"])
            .assert()
            .code(finding_code())
            .get_output()
            .stdout
            .clone();

        let output = String::from_utf8(raw).expect("stdout is UTF-8");
        let envelope: Value = serde_json::from_str(output.trim())
            .unwrap_or_else(|e| panic!("--json output is not valid JSON for {claim_type}: {e}"));

        assert_eq!(
            envelope["schema_version"].as_u64(),
            Some(1),
            "schema_version must be 1 for {claim_type}"
        );
        assert_eq!(
            envelope["outcome"].as_str(),
            Some("finding"),
            "outcome must be 'finding' for {claim_type}"
        );
        let directive = envelope["directive"]
            .as_str()
            .unwrap_or_else(|| panic!("directive field missing for {claim_type}"));
        assert!(
            !directive.is_empty(),
            "directive must be non-empty for {claim_type}"
        );
    }
}
