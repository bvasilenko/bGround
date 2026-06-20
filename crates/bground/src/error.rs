use thiserror::Error;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum BgroundError {
    #[error("claim string is malformed: {0}")]
    ClaimStringMalformed(String),
    #[error("evidence map entry is invalid: {0}")]
    EvidenceMapInvalid(String),
    #[error("unknown claim type: {0}")]
    UnknownClaimType(String),
    #[error("unknown evidence state: {0}")]
    UnknownEvidenceState(String),
    #[error("corpus load failed: {0}")]
    CorpusLoad(String),
    #[error(transparent)]
    Core(#[from] bsuite_core::BsuiteCoreError),
}

impl BgroundError {
    pub fn is_malformed_input(&self) -> bool {
        matches!(
            self,
            Self::ClaimStringMalformed(_) | Self::EvidenceMapInvalid(_) | Self::UnknownClaimType(_)
        )
    }

    pub fn into_core(self) -> bsuite_core::BsuiteCoreError {
        match self {
            Self::Core(e) => e,
            Self::CorpusLoad(msg) => bsuite_core::BsuiteCoreError::CorpusDeserializationFailed(msg),
            other => bsuite_core::BsuiteCoreError::PromptResolution(other.to_string()),
        }
    }
}
