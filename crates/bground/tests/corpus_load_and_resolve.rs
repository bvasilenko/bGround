use bground::{ClaimType, corpus_index::ClaimCorpusIndex};
use bsuite_core::BsuiteCoreError;
use ed25519_dalek::{SigningKey, VerifyingKey};

const CORPUS_TOML: &str = include_str!("../corpus/bground-v0.toml");
const PUBKEY_BYTES: &[u8] = include_bytes!("../corpus/bground-v0-pubkey.bin");
const SIGNKEY_BYTES: &[u8] = include_bytes!("../corpus/bground-v0-signkey.bin");

fn load_verifying_key() -> VerifyingKey {
    let bytes: [u8; 32] = PUBKEY_BYTES.try_into().expect("pubkey is 32 bytes");
    VerifyingKey::from_bytes(&bytes).expect("pubkey is valid")
}

fn load_corpus() -> ClaimCorpusIndex {
    ClaimCorpusIndex::from_toml_signed(CORPUS_TOML, &load_verifying_key())
        .expect("fixture corpus loads cleanly")
}

#[test]
fn corpus_signature_is_valid_against_embedded_pubkey() {
    let _corpus = load_corpus();
}

#[test]
fn all_sixteen_claim_types_are_indexed() {
    let corpus = load_corpus();

    for claim_type in ClaimType::ALL {
        let directive = corpus.resolve(claim_type);
        assert!(
            !directive.as_str().is_empty(),
            "no directive for {claim_type}"
        );
    }
}

#[test]
fn corpus_rejects_wrong_pubkey() {
    let wrong_seed = [0x00u8; 32];
    let wrong_signing_key = SigningKey::from_bytes(&wrong_seed);
    let wrong_pubkey = wrong_signing_key.verifying_key();

    let result = ClaimCorpusIndex::from_toml_signed(CORPUS_TOML, &wrong_pubkey);
    assert!(result.is_err(), "corpus with wrong pubkey must be rejected");
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            bground::BgroundError::Core(BsuiteCoreError::CorpusSignatureInvalid)
        ),
        "wrong-key rejection must be CorpusSignatureInvalid, got: {err}"
    );
}

#[test]
fn corpus_rejects_tampered_directive_content() {
    let tampered = CORPUS_TOML.replacen(
        "Verify the file at the stated path",
        "TAMPERED content here",
        1,
    );

    let result = ClaimCorpusIndex::from_toml_signed(&tampered, &load_verifying_key());
    assert!(
        result.is_err(),
        "tampered corpus content must be rejected by signature check"
    );
}

#[test]
fn pubkey_and_signkey_are_a_matched_pair() {
    let sig_bytes: [u8; 32] = SIGNKEY_BYTES.try_into().expect("signkey is 32 bytes");
    let signing_key = SigningKey::from_bytes(&sig_bytes);
    let derived_pubkey = signing_key.verifying_key();
    let embedded_pubkey = load_verifying_key();

    assert_eq!(
        derived_pubkey.to_bytes(),
        embedded_pubkey.to_bytes(),
        "embedded pubkey must be the verifying key of the embedded signing key"
    );
}

#[test]
fn corpus_rejects_unsigned_appended_entry() {
    // Appending any [[entries]] block changes the bCore-signed payload (routing_key +
    // directive + provenance are signed). The signature check must fire before the index
    // is built; this test verifies that path, not duplicate detection.
    let extra = "\n[[entries]]\nrouting_key = \"bground\"\nclaim_type = \"file-exists\"\ndirective = \"Extra.\"\n[entries.provenance]\nrun_id = \"x\"\niteration = 0\nobservation_source = \"x\"\npre_compliance = 0.0\npost_compliance = 0.0\n";

    let with_extra = format!("{CORPUS_TOML}{extra}");
    let result = ClaimCorpusIndex::from_toml_signed(&with_extra, &load_verifying_key());

    assert!(
        result.is_err(),
        "appended entry must be caught by signature verification"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            bground::BgroundError::Core(BsuiteCoreError::CorpusSignatureInvalid)
        ),
        "appended entry must fail with CorpusSignatureInvalid, got: {err}"
    );
}

#[test]
fn corpus_index_rejects_duplicate_claim_type() {
    // claim_type is not part of the bCore-signed payload, so mutating it does not
    // invalidate the signature. Changing one claim_type to match an existing variant
    // creates a duplicate that must be caught by build_index, not by signature verification.
    let with_duplicate = CORPUS_TOML.replacen(
        "claim_type = \"fn-defined\"",
        "claim_type = \"file-exists\"",
        1,
    );

    let result = ClaimCorpusIndex::from_toml_signed(&with_duplicate, &load_verifying_key());

    assert!(
        result.is_err(),
        "corpus with duplicate claim_type must be rejected by index validation"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, bground::BgroundError::CorpusLoad(_)),
        "duplicate claim_type must produce a CorpusLoad error, got: {err}"
    );
}

#[test]
fn corpus_index_rejects_unknown_claim_type_string() {
    // claim_type is not signed; mutation does not affect signature validity.
    let with_unknown = CORPUS_TOML.replacen(
        "claim_type = \"file-exists\"",
        "claim_type = \"not-a-valid-type\"",
        1,
    );

    let result = ClaimCorpusIndex::from_toml_signed(&with_unknown, &load_verifying_key());

    assert!(
        result.is_err(),
        "corpus with unknown claim_type string must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, bground::BgroundError::CorpusLoad(_)),
        "unknown claim_type must produce a CorpusLoad error, got: {err}"
    );
}

#[test]
fn corpus_index_rejects_empty_claim_type_string() {
    // An empty claim_type is a special case of an unrecognised string; it must
    // be rejected explicitly rather than silently falling through to a missing-variant
    // error at the completeness check.
    let with_empty = CORPUS_TOML.replacen("claim_type = \"file-exists\"", "claim_type = \"\"", 1);

    let result = ClaimCorpusIndex::from_toml_signed(&with_empty, &load_verifying_key());

    assert!(
        result.is_err(),
        "corpus with empty claim_type string must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, bground::BgroundError::CorpusLoad(_)),
        "empty claim_type must produce a CorpusLoad error, got: {err}"
    );
}
