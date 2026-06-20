use assert_cmd::Command;
use bsuite_core::ExitCode;
use httpmock::MockServer;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct TranscriptCase {
    name: &'static str,
    args: &'static [&'static str],
    expected_exit_code: ExitCode,
    directive_emitted: bool,
}

// `update` requires HTTP mock setup; covered by `update_command_writes_transcript_record_regardless_of_server_response`.
const TRANSCRIPT_CASES: &[TranscriptCase] = &[
    TranscriptCase {
        name: "verify-ungrounded",
        args: &["verify", "file-exists:README.md:exists"],
        expected_exit_code: ExitCode::Finding,
        directive_emitted: true,
    },
    TranscriptCase {
        name: "verify-malformed",
        args: &["verify", "unknown-type:target:assertion"],
        expected_exit_code: ExitCode::Usage,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "claim-types",
        args: &["claim-types"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "init",
        args: &["init"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "tail",
        args: &["tail"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
    TranscriptCase {
        name: "explain",
        args: &["explain"],
        expected_exit_code: ExitCode::Success,
        directive_emitted: false,
    },
];

fn bground_command() -> Command {
    Command::cargo_bin("bground").unwrap()
}

fn run_in_transcript_dir(args: &[&str], dir: &TempDir) -> assert_cmd::assert::Assert {
    let mut cmd = bground_command();
    cmd.env("BSUITE_TRANSCRIPT_DIR", dir.path());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert()
}

fn collect_transcript_files(dir: &TempDir) -> Vec<PathBuf> {
    let bground_dir = dir.path().join("bground");
    if !bground_dir.exists() {
        return vec![];
    }
    std::fs::read_dir(&bground_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
        .collect()
}

fn read_transcript_record(path: &Path) -> Value {
    let content = std::fs::read_to_string(path).expect("transcript file is readable UTF-8");
    serde_json::from_str(content.trim()).expect("transcript content is valid JSON")
}

fn read_single_transcript_record(dir: &TempDir) -> Value {
    let files = collect_transcript_files(dir);
    assert_eq!(
        files.len(),
        1,
        "expected exactly one transcript file, found {:?}",
        files
    );
    read_transcript_record(&files[0])
}

#[test]
fn every_command_writes_exactly_one_transcript_record() {
    for case in TRANSCRIPT_CASES {
        let dir = TempDir::new().unwrap();
        run_in_transcript_dir(case.args, &dir).code(case.expected_exit_code.as_i32());

        let files = collect_transcript_files(&dir);
        assert_eq!(
            files.len(),
            1,
            "[{}] expected exactly one transcript file, found {:?}",
            case.name,
            files
        );
    }
}

#[test]
fn transcript_records_carry_correct_exit_code_and_directive_emitted_flag() {
    for case in TRANSCRIPT_CASES {
        let dir = TempDir::new().unwrap();
        run_in_transcript_dir(case.args, &dir).code(case.expected_exit_code.as_i32());

        let record = read_single_transcript_record(&dir);

        assert_eq!(
            record["exit_code"].as_u64(),
            Some(case.expected_exit_code.as_i32() as u64),
            "[{}] transcript exit_code must match process exit code",
            case.name
        );
        assert_eq!(
            record["directive_emitted"].as_bool(),
            Some(case.directive_emitted),
            "[{}] transcript directive_emitted field",
            case.name
        );
    }
}

#[test]
fn transcript_schema_carries_all_required_fields() {
    let dir = TempDir::new().unwrap();
    run_in_transcript_dir(&["verify", "file-exists:README.md:exists"], &dir)
        .code(ExitCode::Finding.as_i32());

    let record = read_single_transcript_record(&dir);

    assert_eq!(record["schema_version"].as_u64(), Some(1), "schema_version");
    assert_eq!(
        record["binary_name"].as_str(),
        Some("bground"),
        "binary_name"
    );
    assert!(
        !record["binary_version"].as_str().unwrap_or("").is_empty(),
        "binary_version must be non-empty"
    );
    assert!(
        !record["invocation_id"].as_str().unwrap_or("").is_empty(),
        "invocation_id must be non-empty"
    );
    assert!(
        !record["timestamp"].as_str().unwrap_or("").is_empty(),
        "timestamp must be non-empty"
    );
    assert_eq!(
        record["routing_key"].as_str(),
        Some("bground"),
        "routing_key"
    );
    assert!(
        record["host_context"].as_str().is_some(),
        "host_context must be a string"
    );
    assert!(
        record["exit_code"].as_u64().is_some(),
        "exit_code must be a number"
    );
    assert!(
        record["directive_emitted"].as_bool().is_some(),
        "directive_emitted must be a bool"
    );
    assert!(
        record["elapsed_ms"].as_u64().is_some(),
        "elapsed_ms must be a number"
    );
    assert_eq!(record["corpus_version"].as_u64(), Some(1), "corpus_version");
    assert!(
        record["additional_fields"].as_object().is_some(),
        "additional_fields must be an object"
    );
}

#[test]
fn consecutive_invocations_produce_distinct_invocation_ids() {
    let dir = TempDir::new().unwrap();

    for _ in 0..3 {
        run_in_transcript_dir(&["claim-types"], &dir).success();
    }

    let files = collect_transcript_files(&dir);
    assert_eq!(
        files.len(),
        3,
        "three invocations must produce three separate transcript files"
    );

    let ids: Vec<String> = files
        .iter()
        .map(|p| {
            let record = read_transcript_record(p);
            record["invocation_id"].as_str().unwrap().to_owned()
        })
        .collect();

    let unique_count = ids.iter().collect::<std::collections::BTreeSet<_>>().len();
    assert_eq!(
        unique_count, 3,
        "each invocation must carry a unique invocation_id; got: {:?}",
        ids
    );
}

#[test]
fn update_command_writes_transcript_record_regardless_of_server_response() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.path("/manifest.json");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"schema_version":1,"binary_name":"bground","version":"0.1.0","release_at":"2025-01-01T00:00:00Z","platforms":{},"corpus_version":1,"obfuscation_tier":"none","signing_key_id":"fixture"}"#);
    });

    let dir = TempDir::new().unwrap();
    bground_command()
        .env("BSUITE_TRANSCRIPT_DIR", dir.path())
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .failure();

    let record = read_single_transcript_record(&dir);
    let exit_code = record["exit_code"]
        .as_u64()
        .expect("exit_code must be present in transcript");
    assert_ne!(
        exit_code, 0,
        "failed update must record a non-zero exit_code"
    );
}
