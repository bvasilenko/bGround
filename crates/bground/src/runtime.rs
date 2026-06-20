use bground::corpus_index::ClaimCorpusIndex;
use bsuite_core::{
    BinaryDefaults, BsuiteCoreError, FileSystemManifestOverlayReader, FileSystemTranscriptAppender,
    FullAdapterHostBinder, HostContext, HostInvocationContext, ManifestOverlay,
    ManifestOverlayReader,
};
use std::path::{Path, PathBuf};

const CORPUS_TOML: &str = include_str!("../corpus/bground-v0.toml");
const PUBKEY_BYTES: &[u8] = include_bytes!("../corpus/bground-v0-pubkey.bin");

pub const EMBEDDED_CORPUS_VERSION: u32 = 1;

pub struct BinaryRuntime {
    pub corpus: ClaimCorpusIndex,
    pub appender: FileSystemTranscriptAppender,
    pub host_context: HostContext,
    pub invocation_context: Option<HostInvocationContext>,
    pub install_dir: PathBuf,
    pub corpus_version: u32,
}

impl BinaryRuntime {
    pub fn init(install_dir: PathBuf) -> Result<Self, BsuiteCoreError> {
        let corpus = load_corpus()?;
        let defaults = load_defaults(&install_dir)?;
        let appender = FileSystemTranscriptAppender::from_base_dir(
            defaults.transcript_dir,
            defaults.transcript_retention_days,
        );
        let host_binder = FullAdapterHostBinder::from_env()?;
        Ok(Self {
            corpus,
            appender,
            host_context: host_binder.resolved_host_context(),
            invocation_context: host_binder.invocation_context().cloned(),
            install_dir,
            corpus_version: EMBEDDED_CORPUS_VERSION,
        })
    }
}

fn load_corpus() -> Result<ClaimCorpusIndex, BsuiteCoreError> {
    let pubkey = load_pubkey()?;
    ClaimCorpusIndex::from_toml_signed(CORPUS_TOML, &pubkey)
        .map_err(|e| BsuiteCoreError::CorpusDeserializationFailed(e.to_string()))
}

fn load_pubkey() -> Result<ed25519_dalek::VerifyingKey, BsuiteCoreError> {
    let bytes: [u8; 32] = PUBKEY_BYTES.try_into().map_err(|_| {
        BsuiteCoreError::CorpusDeserializationFailed("embedded pubkey is not 32 bytes".to_owned())
    })?;
    ed25519_dalek::VerifyingKey::from_bytes(&bytes)
        .map_err(|e| BsuiteCoreError::CorpusDeserializationFailed(e.to_string()))
}

fn load_defaults(install_dir: &Path) -> Result<BinaryDefaults, BsuiteCoreError> {
    let base_dir = FileSystemTranscriptAppender::new("bground")?
        .directory()
        .to_path_buf();
    let overlay_reader = FileSystemManifestOverlayReader::new("bground", install_dir);
    let overlay = overlay_reader
        .read()
        .unwrap_or_else(|_| ManifestOverlay::empty());
    let mut defaults = BinaryDefaults {
        transcript_retention_days: env_transcript_retention_days(),
        transcript_dir: base_dir,
        corpus_dir: install_dir.to_path_buf(),
        update_check_interval_minutes: 60,
        stdout_byte_cap: 65536,
        binary_timeout_ms: 5000,
    };
    overlay.merge_into_defaults(&mut defaults);
    Ok(defaults)
}

fn env_transcript_retention_days() -> u32 {
    std::env::var("BSUITE_TRANSCRIPT_RETENTION_DAYS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(90)
}
