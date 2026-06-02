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
    #[error(transparent)]
    Core(#[from] bsuite_core::BsuiteCoreError),
}
