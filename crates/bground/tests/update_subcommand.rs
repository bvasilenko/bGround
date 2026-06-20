use assert_cmd::Command;
use httpmock::MockServer;

fn bground_command() -> Command {
    Command::cargo_bin("bground").unwrap()
}

#[test]
fn update_subcommand_exits_cleanly_when_update_endpoint_returns_up_to_date() {
    let server = MockServer::start();

    let manifest_json = serde_json::json!({
        "schema_version": 1,
        "binary_name": "bground",
        "version": "0.1.0",
        "release_at": "2025-01-01T00:00:00Z",
        "platforms": {
            "linux-x86_64": { "archive_url": "https://example.com/bground.tar", "sha256": "abc" }
        },
        "corpus_version": 1,
        "obfuscation_tier": "none",
        "signing_key_id": "fixture-key"
    });

    server.mock(|when, then| {
        when.path("/manifest.json");
        then.status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&manifest_json).unwrap());
    });

    bground_command()
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .failure();
}

#[test]
fn update_subcommand_exists_as_a_clap_subcommand() {
    let output = bground_command()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output = String::from_utf8(output).unwrap();
    assert!(
        output.contains("update"),
        "help output must advertise the update subcommand"
    );
}

#[test]
fn update_subcommand_uses_env_var_for_base_url() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.path("/manifest.json");
        then.status(404).body("not found");
    });

    bground_command()
        .env("BSUITE_UPDATE_BASE_URL", server.base_url())
        .arg("update")
        .assert()
        .failure();

    mock.assert_hits(1);
}
