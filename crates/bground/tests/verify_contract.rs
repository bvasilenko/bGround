mod common;

use bground::corpus_index::ClaimCorpusIndex;
use bground::{ClaimString, ClaimType};
use bsuite_core::ExitCode;
use ed25519_dalek::VerifyingKey;

const CORPUS_TOML: &str = include_str!("../corpus/bground-v0.toml");
const PUBKEY_BYTES: &[u8] = include_bytes!("../corpus/bground-v0-pubkey.bin");

fn load_corpus() -> ClaimCorpusIndex {
    let bytes: [u8; 32] = PUBKEY_BYTES.try_into().expect("pubkey is 32 bytes");
    let pubkey = VerifyingKey::from_bytes(&bytes).expect("pubkey is valid");
    ClaimCorpusIndex::from_toml_signed(CORPUS_TOML, &pubkey).expect("fixture corpus loads cleanly")
}

#[test]
fn every_claim_type_resolves_to_a_non_empty_directive() {
    let corpus = load_corpus();

    for claim_type in ClaimType::ALL {
        let directive = corpus.resolve(claim_type);
        assert!(
            !directive.as_str().is_empty(),
            "empty directive for {claim_type}"
        );
        common::assert_corpus_directive(directive.as_str());
    }
}

#[test]
fn all_claim_types_produce_distinct_directives() {
    let corpus = load_corpus();

    let directives: Vec<&str> = ClaimType::ALL
        .iter()
        .map(|ct| corpus.resolve(*ct).as_str())
        .collect();

    for (i, a) in directives.iter().enumerate() {
        for (j, b) in directives.iter().enumerate() {
            if i != j {
                assert_ne!(
                    a, b,
                    "claim types at index {i} and {j} share the same directive"
                );
            }
        }
    }
}

#[test]
fn finding_exit_code_is_returned_for_every_valid_claim_type() {
    let corpus = load_corpus();
    let host_context = bsuite_core::HostContext::L2a;

    for claim_type in ClaimType::ALL {
        let claim = format!("{claim_type}:target:assertion");
        let args = bground::VerifyArgs {
            claim: claim.clone(),
            evidence: vec![],
            manifest: None,
            json: false,
            quiet: false,
            reason: None,
        };

        let (_, exit_code) = bground::verify::run(&args, &corpus, host_context)
            .unwrap_or_else(|e| panic!("verify failed for {claim_type}: {e}"));

        assert_eq!(
            exit_code,
            ExitCode::Finding,
            "unexpected exit code for {claim_type}"
        );
    }
}

#[test]
fn verify_rejects_malformed_claim_string_with_bground_error() {
    let corpus = load_corpus();
    let host_context = bsuite_core::HostContext::L2a;

    let args = bground::VerifyArgs {
        claim: "not-a-valid-type:target:assertion".to_owned(),
        evidence: vec![],
        manifest: None,
        json: false,
        quiet: false,
        reason: None,
    };

    let err = bground::verify::run(&args, &corpus, host_context).unwrap_err();
    assert!(
        err.is_malformed_input(),
        "unknown claim type must be reported as malformed input, got: {err}"
    );
}

#[test]
fn verify_accepts_non_empty_evidence_for_all_claim_types() {
    // This test is generalised across all 16 variants so it catches regressions where
    // a future claim type is added with different evidence-parsing rules.
    let corpus = load_corpus();
    let host_context = bsuite_core::HostContext::L2a;

    for claim_type in ClaimType::ALL {
        let claim = format!("{claim_type}:target:assertion");
        let args = bground::VerifyArgs {
            claim: claim.clone(),
            evidence: vec![("key".to_owned(), "value".to_owned())],
            manifest: None,
            json: false,
            quiet: false,
            reason: None,
        };

        let result = bground::verify::run(&args, &corpus, host_context);
        assert!(
            result.is_ok(),
            "non-empty evidence must not cause an error for {claim_type}"
        );
    }
}

#[test]
fn corpus_load_error_is_not_classified_as_malformed_input() {
    // CorpusLoad failures are internal startup errors, not agent-supplied bad input.
    // is_malformed_input() must return false so the process exits 2 (InternalError),
    // not 64 (Usage), when corpus loading fails.
    let err = bground::BgroundError::CorpusLoad("test failure".to_owned());
    assert!(
        !err.is_malformed_input(),
        "CorpusLoad must not be classified as malformed input"
    );
}
