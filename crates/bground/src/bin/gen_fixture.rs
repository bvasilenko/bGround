use base64::Engine;
use bsuite_core::{CorpusEntry, CorpusFile, ProvenanceRecord, RoutingKey};
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use std::{fs, path::Path};

/// Fixed seed so the fixture keypair is reproducible across machines.
/// This is fixture-only material; it must never be used for production signing.
const FIXTURE_SEED: [u8; 32] = [
    0x62, 0x67, 0x72, 0x6f, 0x75, 0x6e, 0x64, 0x2d, 0x66, 0x69, 0x78, 0x74, 0x75, 0x72, 0x65, 0x2d,
    0x76, 0x30, 0x2d, 0x73, 0x65, 0x65, 0x64, 0x2d, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
];

const CANONICAL_KEY_ID: &str = "bground-fixture-v0";
const OBSERVATION_SOURCE: &str = "hand-authored-fixture-v0";
const RUN_ID: &str = "hand-authored-fixture-v0";

struct EntrySpec {
    claim_type: &'static str,
    directive: &'static str,
}

const ENTRIES: &[EntrySpec] = &[
    EntrySpec {
        claim_type: "file-exists",
        directive: "Verify the file at the stated path exists in the working tree before asserting \
its presence. Run a stat or directory-listing check against the exact path supplied as evidence; \
do not infer presence from a recent creation step without re-reading the filesystem. If the file \
is absent, name the full expected path and either create it or retract the claim before proceeding. \
Do not treat cached or in-memory knowledge of the filesystem as current.",
    },
    EntrySpec {
        claim_type: "fn-defined",
        directive: "Locate the function definition in the exact source file supplied as evidence \
before asserting it is defined. Search by the verbatim function name at the file scope; a type \
method, macro expansion, or trait implementation is not a top-level definition unless the claim \
explicitly targets one of those forms. If the function is absent, name the file and the nearest \
logical insertion point before asserting definition. Do not assert from a prior search result \
without confirming the file has not changed.",
    },
    EntrySpec {
        claim_type: "fn-signature",
        directive: "Read the complete function signature at the declared source location before \
asserting it matches the claim. Signature match requires all of: parameter names, parameter types, \
return type, visibility modifier, and any generic bounds. A partial field match is not a match; \
name every differing field. Do not assert a signature from memory or from a recent edit without \
re-reading the file to confirm the edit landed as intended.",
    },
    EntrySpec {
        claim_type: "value-equals",
        directive: "Read the value from its authoritative source -- the file, environment variable, \
configuration row, or constant definition named in the evidence -- before asserting equality. Do \
not assert from memory of a prior value because values change during a session. If the observed \
value differs from the asserted value, state both explicitly and either correct the source or \
retract the assertion before proceeding.",
    },
    EntrySpec {
        claim_type: "dependency-installed",
        directive: "Verify the dependency appears in the package manifest at the exact version or \
version range stated in the assertion. Check the manifest file supplied as evidence; do not rely \
on a prior install step having succeeded without confirming the manifest was updated. If the \
dependency is absent or at a different version, name the manifest path, the expected entry, and \
the corrective action before asserting installation.",
    },
    EntrySpec {
        claim_type: "state-equals",
        directive: "Read the current state from the system component named in the evidence before \
asserting it equals the claimed value. State is runtime data and changes independently of source \
code. Confirm through a direct query, log line, or test assertion, not through inference from how \
the code is written. If the state differs from the assertion, report the observed value and stop \
until the discrepancy is resolved.",
    },
    EntrySpec {
        claim_type: "fn-return-type",
        directive: "Trace the return type of the function from its declaration through any type \
aliases, generics, or conditional compilation guards before asserting a return type. Declared \
return type and effective return type may differ when aliases, newtypes, or feature flags are \
involved. Name the declaration site, any aliasing chain, and the terminal resolved type. Do not \
assert from the call site alone.",
    },
    EntrySpec {
        claim_type: "url-returns",
        directive: "Make the HTTP request at the stated URL and examine the actual response status, \
headers, and body before asserting what the endpoint returns. Do not assert response shape from \
documentation, previous calls, or code reading alone because live endpoints may differ from their \
specs. Record the exact status code and the response body fragment that confirms or refutes the \
assertion.",
    },
    EntrySpec {
        claim_type: "cmd-output-matches",
        directive: "Run the exact command in the evidence and capture its stdout before asserting \
the output matches the claim. Do not assert from a prior run because command output changes when \
the environment, input files, or code change. Quote the relevant output lines verbatim and confirm \
that the match is exact rather than approximate.",
    },
    EntrySpec {
        claim_type: "behavior",
        directive: "Exercise the behavior through a direct invocation, test execution, or log \
observation before asserting it holds. Behavioral claims derived solely from reading source code \
are hypotheses, not verified evidence. If the observed behavior differs from the assertion, report \
the observed output and the code path that produced it; do not assert the behavior until the \
discrepancy is investigated.",
    },
    EntrySpec {
        claim_type: "coherent-refactor",
        directive: "Verify that every call site, type reference, and module import that the \
refactored entity touches compiles and behaves identically after the change before asserting \
coherence. A refactor is coherent when no behavior observable at a public boundary changes. Check \
every caller listed in the evidence; if any caller is missing, name it and confirm it compiles \
before marking the refactor coherent.",
    },
    EntrySpec {
        claim_type: "coherent-migration",
        directive: "Confirm that every data record, configuration value, and external dependency \
the migration transforms satisfies the post-migration schema before asserting migration coherence. \
Test with a representative record from the evidence set, not just the schema definition. If any \
record fails post-migration validation, report the failing record and the schema field that rejects \
it before asserting coherence.",
    },
    EntrySpec {
        claim_type: "coherent-spec-impl",
        directive: "Read the specification section named in the evidence and compare each normative \
requirement against the implementation before asserting spec alignment. Partial compliance is not \
coherence; every MUST and SHOULD clause in the referenced section must be accounted for. If a \
requirement is unimplemented, name the clause and the implementation gap before asserting full \
coherence.",
    },
    EntrySpec {
        claim_type: "coherent-contract",
        directive: "Verify that the implementation satisfies the contract's preconditions, \
postconditions, and invariants through test execution or formal analysis of the evidence before \
asserting contract coherence. Reading the implementation and judging it likely to satisfy the \
contract is not sufficient. If any contract clause fails, name the clause and the observed \
violation before asserting coherence.",
    },
    EntrySpec {
        claim_type: "coherent-test-cover",
        directive: "Run the test suite against the code under assertion and examine the coverage \
output before claiming adequate test coverage. Do not infer coverage from the presence of test \
files because files that exist but do not execute the code path in question do not contribute \
coverage. Name the specific code paths exercised by the tests in the evidence and confirm no \
critical path is excluded.",
    },
    EntrySpec {
        claim_type: "coherent-dep-upgrade",
        directive: "Build and run the full test suite with the upgraded dependency version before \
asserting a dependency upgrade is coherent. A version bump that compiles but does not pass all \
tests is not coherent. If any test fails after the upgrade, report the failure, the breaking \
change in the dependency changelog, and the corrective action needed before the upgrade can \
proceed.",
    },
];

/// Includes the bGround-local `claim_type` discriminator field that bCore does not know about.
#[derive(Serialize)]
struct FixtureFile {
    schema_version: u32,
    signature: String,
    canonical_key_id: &'static str,
    entries: Vec<FixtureEntry>,
}

#[derive(Serialize)]
struct FixtureEntry {
    routing_key: RoutingKey,
    claim_type: &'static str,
    directive: &'static str,
    provenance: FixtureProvenance,
}

#[derive(Serialize)]
struct FixtureProvenance {
    run_id: &'static str,
    iteration: u32,
    observation_source: &'static str,
    pre_compliance: f64,
    post_compliance: f64,
}

fn main() {
    let signing_key = SigningKey::from_bytes(&FIXTURE_SEED);
    let verifying_key = signing_key.verifying_key();

    let core_entries: Vec<CorpusEntry> = ENTRIES
        .iter()
        .map(|spec| CorpusEntry {
            routing_key: RoutingKey::BGround,
            directive: spec.directive.to_owned(),
            provenance: ProvenanceRecord {
                run_id: RUN_ID.to_owned(),
                iteration: 0,
                observation_source: OBSERVATION_SOURCE.to_owned(),
                pre_compliance: 0.0,
                post_compliance: 0.0,
            },
        })
        .collect();

    let mut corpus = CorpusFile {
        schema_version: 1,
        signature: String::new(),
        canonical_key_id: CANONICAL_KEY_ID.to_owned(),
        entries: core_entries,
    };

    let payload_bytes = bsuite_core::corpus::canonical_payload_bytes(&corpus)
        .expect("fixture provenance scores are finite; canonicalization must succeed");

    let signature = signing_key.sign(&payload_bytes);
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    corpus.signature = format!("ed25519:{sig_b64}");

    let fixture_entries: Vec<FixtureEntry> = ENTRIES
        .iter()
        .map(|spec| FixtureEntry {
            routing_key: RoutingKey::BGround,
            claim_type: spec.claim_type,
            directive: spec.directive,
            provenance: FixtureProvenance {
                run_id: RUN_ID,
                iteration: 0,
                observation_source: OBSERVATION_SOURCE,
                pre_compliance: 0.0,
                post_compliance: 0.0,
            },
        })
        .collect();

    let fixture_file = FixtureFile {
        schema_version: 1,
        signature: corpus.signature.clone(),
        canonical_key_id: CANONICAL_KEY_ID,
        entries: fixture_entries,
    };

    let header = "# Fixture corpus. Hand-authored seed material until an evolved corpus ships at a later cycle. Not for production trust.\n\n";
    let body = toml::to_string_pretty(&fixture_file).expect("fixture file serialises cleanly");
    let toml_content = format!("{header}{body}");

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus");
    fs::create_dir_all(&out_dir).expect("create corpus directory");

    fs::write(out_dir.join("bground-v0.toml"), toml_content).expect("write corpus TOML");
    fs::write(
        out_dir.join("bground-v0-pubkey.bin"),
        verifying_key.to_bytes(),
    )
    .expect("write verifying key");
    fs::write(
        out_dir.join("bground-v0-signkey.bin"),
        signing_key.to_bytes(),
    )
    .expect("write signing key");

    println!("Corpus files written to {}", out_dir.display());
}
