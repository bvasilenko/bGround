#![allow(dead_code)]

// Shared with `stdout_primacy_contract` so both test files exercise identical inputs.
pub const MALFORMED_CLAIM_SHAPES: &[&str] = &[
    "",
    "file-exists",
    "file-exists:target",
    ":target:assertion",
    "file-exists::assertion",
    "file-exists:target:",
];

// Derived from `ClaimType::EXCLUDED_CANDIDATES` so coverage automatically extends
// to any newly added excluded names without touching individual test files.
pub fn malformed_claim_type_claims() -> Vec<String> {
    let mut claims = vec!["unknown-type:target:assertion".to_owned()];
    claims.extend(
        bground::ClaimType::EXCLUDED_CANDIDATES
            .iter()
            .map(|name| format!("{name}:target:assertion")),
    );
    claims
}

pub fn assert_public_voice_safe_runtime_output(output: &str) {
    for prohibited in prohibited_runtime_phrases() {
        assert!(
            !output.contains(&prohibited),
            "runtime output contains prohibited public-voice phrase: {prohibited}"
        );
    }
}

pub fn assert_corpus_directive(output: &str) {
    assert_public_voice_safe_runtime_output(output);
    assert!(!output.is_empty(), "corpus directive must be non-empty");
    assert!(
        !output.contains("[bground placeholder directive"),
        "corpus-backed output must not contain the placeholder header"
    );
}

fn prohibited_runtime_phrases() -> Vec<String> {
    vec![["0.1.0", "-skeleton"].concat(), ["v0.2", "+"].concat()]
}
