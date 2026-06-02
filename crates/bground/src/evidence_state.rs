use crate::BgroundError;
use bsuite_core::ExitCode;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EvidenceState {
    #[serde(rename = "grounded")]
    Grounded,
    #[serde(rename = "ungrounded")]
    Ungrounded,
    #[serde(rename = "malformed")]
    Malformed,
}

impl EvidenceState {
    pub const ALL: [Self; 3] = [Self::Grounded, Self::Ungrounded, Self::Malformed];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::Grounded => "grounded",
            Self::Ungrounded => "ungrounded",
            Self::Malformed => "malformed",
        }
    }
}

impl fmt::Display for EvidenceState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.stable_name())
    }
}

impl FromStr for EvidenceState {
    type Err = BgroundError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "grounded" => Ok(Self::Grounded),
            "ungrounded" => Ok(Self::Ungrounded),
            "malformed" => Ok(Self::Malformed),
            other => Err(BgroundError::UnknownEvidenceState(other.to_owned())),
        }
    }
}

impl From<EvidenceState> for ExitCode {
    fn from(value: EvidenceState) -> Self {
        match value {
            EvidenceState::Grounded => Self::Success,
            EvidenceState::Ungrounded => Self::Finding,
            EvidenceState::Malformed => Self::Usage,
        }
    }
}
