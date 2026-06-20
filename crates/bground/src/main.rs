use bground::corpus_index::ClaimCorpusIndex;
use bground::{BgroundCli, BgroundError, ClaimType, Cmd};
use bsuite_core::{
    BsuiteCoreError, EmitFormat, ExitCode, FileSystemManifestOverlayReader,
    FileSystemTranscriptAppender, FullAdapterHostBinder, HostContext, ManifestOverlayReader,
    ProcessExitEmitter, TranscriptAppender, TranscriptRecord, format_context_tag,
    prompt_resolver::DirectiveString,
};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use ulid::Ulid;

const CORPUS_TOML: &str = include_str!("../corpus/bground-v0.toml");
const PUBKEY_BYTES: &[u8] = include_bytes!("../corpus/bground-v0-pubkey.bin");

fn main() {
    let started_at = Instant::now();
    let cli = BgroundCli::parse();

    let format = if cmd_wants_json(&cli.cmd) {
        EmitFormat::Json
    } else {
        EmitFormat::Plain
    };
    let mut emitter = ProcessExitEmitter::new(format);

    let result = run(cli, started_at);
    let exit_code = dispatch_to_emitter(result, &mut emitter);
    std::process::exit(exit_code.as_i32());
}

fn run(cli: BgroundCli, started_at: Instant) -> Result<(DirectiveString, ExitCode), RunError> {
    let pubkey = load_pubkey()?;

    let corpus = ClaimCorpusIndex::from_toml_signed(CORPUS_TOML, &pubkey)
        .map_err(|e| RunError::Internal(bground_to_core(e)))?;

    let install_dir = install_dir();
    let overlay_reader = FileSystemManifestOverlayReader::new("bground", &install_dir);
    let _overlay = overlay_reader
        .read()
        .unwrap_or_else(|_| bsuite_core::ManifestOverlay::empty());

    let host_binder = FullAdapterHostBinder::from_env().map_err(RunError::Internal)?;
    let host_context = host_binder.resolved_host_context();

    let appender = FileSystemTranscriptAppender::new("bground").map_err(RunError::Internal)?;

    match cli.cmd {
        Cmd::Verify(args) => {
            let outcome = bground::verify::run(&args, &corpus, host_context).map_err(|e| {
                if e.is_malformed_input() {
                    RunError::Malformed(e)
                } else {
                    RunError::Internal(bground_to_core(e))
                }
            })?;

            let (directive, exit_code) = outcome;

            append_transcript(
                &appender,
                host_context,
                exit_code,
                true,
                host_binder.invocation_context(),
                started_at,
            );

            Ok((directive, exit_code))
        }

        Cmd::ClaimTypes => {
            for claim_type in ClaimType::ALL {
                println!("{claim_type}");
            }
            append_transcript(
                &appender,
                host_context,
                ExitCode::Success,
                false,
                host_binder.invocation_context(),
                started_at,
            );
            Ok((DirectiveString::new(String::new()), ExitCode::Success))
        }

        Cmd::Update => {
            bground::update::run(&install_dir)
                .map_err(|e| RunError::Internal(bground_to_core(e)))?;
            Ok((DirectiveString::new(String::new()), ExitCode::Success))
        }

        Cmd::Init | Cmd::Tail | Cmd::Explain => {
            Ok((DirectiveString::new(String::new()), ExitCode::Success))
        }
    }
}

fn bground_to_core(e: BgroundError) -> BsuiteCoreError {
    match e {
        BgroundError::Core(core_err) => core_err,
        BgroundError::CorpusLoad(msg) => BsuiteCoreError::CorpusDeserializationFailed(msg),
        other => BsuiteCoreError::PromptResolution(other.to_string()),
    }
}

fn load_pubkey() -> Result<ed25519_dalek::VerifyingKey, RunError> {
    let bytes: [u8; 32] = PUBKEY_BYTES.try_into().map_err(|_| {
        RunError::Internal(BsuiteCoreError::CorpusDeserializationFailed(
            "embedded pubkey is not 32 bytes".to_owned(),
        ))
    })?;
    ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|e| {
        RunError::Internal(BsuiteCoreError::CorpusDeserializationFailed(e.to_string()))
    })
}

fn install_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn append_transcript(
    appender: &FileSystemTranscriptAppender,
    host_context: HostContext,
    exit_code: ExitCode,
    directive_emitted: bool,
    invocation_context: Option<&bsuite_core::HostInvocationContext>,
    started_at: Instant,
) {
    let context_tag = invocation_context.map(format_context_tag);
    let additional_fields = match context_tag {
        Some(tag) => serde_json::json!({ "context_tag": tag }),
        None => serde_json::json!({}),
    };

    let record = TranscriptRecord {
        schema_version: 1,
        binary_name: "bground".to_owned(),
        binary_version: env!("CARGO_PKG_VERSION").to_owned(),
        invocation_id: Ulid::new().to_string(),
        timestamp: chrono::Utc::now(),
        routing_key: bground::routing_key(),
        host_context,
        exit_code: exit_code.as_i32() as u8,
        directive_emitted,
        elapsed_ms: started_at.elapsed().as_millis() as u64,
        corpus_version: 1,
        additional_fields,
    };

    let _ = appender.append(&record);
}

fn cmd_wants_json(cmd: &Cmd) -> bool {
    matches!(cmd, Cmd::Verify(args) if args.json)
}

enum RunError {
    Malformed(BgroundError),
    Internal(BsuiteCoreError),
}

fn dispatch_to_emitter(
    result: Result<(DirectiveString, ExitCode), RunError>,
    emitter: &mut ProcessExitEmitter,
) -> ExitCode {
    match result {
        Ok((directive, exit_code)) => {
            if !directive.as_str().is_empty() {
                emitter.emit_directive(Ok((directive, exit_code)))
            } else {
                exit_code
            }
        }
        Err(RunError::Malformed(e)) => {
            eprintln!("{e}");
            ExitCode::Usage
        }
        Err(RunError::Internal(e)) => emitter.emit_directive(Err(e)),
    }
}
