use bsuite_core::{
    ExitCode, HostContext, HostInvocationContext, TranscriptAppender, TranscriptRecord,
    format_context_tag,
};
use std::time::Instant;
use ulid::Ulid;

pub struct InvocationTranscript {
    invocation_id: String,
    started_at: Instant,
    host_context: HostContext,
    invocation_context: Option<HostInvocationContext>,
    corpus_version: u32,
}

impl InvocationTranscript {
    pub fn start(
        host_context: HostContext,
        invocation_context: Option<HostInvocationContext>,
        corpus_version: u32,
    ) -> Self {
        Self {
            invocation_id: Ulid::new().to_string(),
            started_at: Instant::now(),
            host_context,
            invocation_context,
            corpus_version,
        }
    }

    pub fn flush(
        &self,
        appender: &impl TranscriptAppender,
        exit_code: ExitCode,
        directive_emitted: bool,
    ) {
        let additional_fields = match &self.invocation_context {
            Some(ctx) => serde_json::json!({ "context_tag": format_context_tag(ctx) }),
            None => serde_json::json!({}),
        };

        let record = TranscriptRecord {
            schema_version: 1,
            binary_name: env!("CARGO_PKG_NAME").to_owned(),
            binary_version: env!("CARGO_PKG_VERSION").to_owned(),
            invocation_id: self.invocation_id.clone(),
            timestamp: chrono::Utc::now(),
            routing_key: bground::routing_key(),
            host_context: self.host_context,
            exit_code: exit_code.as_i32() as u8,
            directive_emitted,
            elapsed_ms: self.started_at.elapsed().as_millis() as u64,
            corpus_version: self.corpus_version,
            additional_fields,
        };

        let _ = appender.append(&record);
    }
}
