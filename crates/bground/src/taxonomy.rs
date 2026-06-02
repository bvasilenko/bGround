use crate::BgroundError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Supported claim taxonomy for this version.
///
/// Candidate names: FileExists, FnDefined, FnSignature, ValueEquals,
/// DependencyInstalled, StateEquals, FnReturnType, UrlReturns,
/// CmdOutputMatches, Behavior, CoherentRefactor, CoherentMigration,
/// CoherentSpecImpl, CoherentContract, CoherentTestCover, CoherentDepUpgrade,
/// CoherentRunbookAlignment, CoherentPerfBaseline.
///
/// Selected names: FileExists, FnDefined, FnSignature, ValueEquals,
/// DependencyInstalled, StateEquals, FnReturnType, UrlReturns,
/// CmdOutputMatches, Behavior, CoherentRefactor, CoherentMigration,
/// CoherentSpecImpl, CoherentContract, CoherentTestCover, CoherentDepUpgrade.
///
/// Excluded names: CoherentRunbookAlignment, CoherentPerfBaseline.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ClaimType {
    #[serde(rename = "file-exists")]
    FileExists,
    #[serde(rename = "fn-defined")]
    FnDefined,
    #[serde(rename = "fn-signature")]
    FnSignature,
    #[serde(rename = "value-equals")]
    ValueEquals,
    #[serde(rename = "dependency-installed")]
    DependencyInstalled,
    #[serde(rename = "state-equals")]
    StateEquals,
    #[serde(rename = "fn-return-type")]
    FnReturnType,
    #[serde(rename = "url-returns")]
    UrlReturns,
    #[serde(rename = "cmd-output-matches")]
    CmdOutputMatches,
    #[serde(rename = "behavior")]
    Behavior,
    #[serde(rename = "coherent-refactor")]
    CoherentRefactor,
    #[serde(rename = "coherent-migration")]
    CoherentMigration,
    #[serde(rename = "coherent-spec-impl")]
    CoherentSpecImpl,
    #[serde(rename = "coherent-contract")]
    CoherentContract,
    #[serde(rename = "coherent-test-cover")]
    CoherentTestCover,
    #[serde(rename = "coherent-dep-upgrade")]
    CoherentDepUpgrade,
}

impl ClaimType {
    pub const ALL: [Self; 16] = [
        Self::FileExists,
        Self::FnDefined,
        Self::FnSignature,
        Self::ValueEquals,
        Self::DependencyInstalled,
        Self::StateEquals,
        Self::FnReturnType,
        Self::UrlReturns,
        Self::CmdOutputMatches,
        Self::Behavior,
        Self::CoherentRefactor,
        Self::CoherentMigration,
        Self::CoherentSpecImpl,
        Self::CoherentContract,
        Self::CoherentTestCover,
        Self::CoherentDepUpgrade,
    ];

    pub const EXCLUDED_CANDIDATES: [&'static str; 2] =
        ["coherent-runbook-alignment", "coherent-perf-baseline"];

    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::FileExists => "file-exists",
            Self::FnDefined => "fn-defined",
            Self::FnSignature => "fn-signature",
            Self::ValueEquals => "value-equals",
            Self::DependencyInstalled => "dependency-installed",
            Self::StateEquals => "state-equals",
            Self::FnReturnType => "fn-return-type",
            Self::UrlReturns => "url-returns",
            Self::CmdOutputMatches => "cmd-output-matches",
            Self::Behavior => "behavior",
            Self::CoherentRefactor => "coherent-refactor",
            Self::CoherentMigration => "coherent-migration",
            Self::CoherentSpecImpl => "coherent-spec-impl",
            Self::CoherentContract => "coherent-contract",
            Self::CoherentTestCover => "coherent-test-cover",
            Self::CoherentDepUpgrade => "coherent-dep-upgrade",
        }
    }
}

impl fmt::Display for ClaimType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.stable_name())
    }
}

impl FromStr for ClaimType {
    type Err = BgroundError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "file-exists" => Ok(Self::FileExists),
            "fn-defined" => Ok(Self::FnDefined),
            "fn-signature" => Ok(Self::FnSignature),
            "value-equals" => Ok(Self::ValueEquals),
            "dependency-installed" => Ok(Self::DependencyInstalled),
            "state-equals" => Ok(Self::StateEquals),
            "fn-return-type" => Ok(Self::FnReturnType),
            "url-returns" => Ok(Self::UrlReturns),
            "cmd-output-matches" => Ok(Self::CmdOutputMatches),
            "behavior" => Ok(Self::Behavior),
            "coherent-refactor" => Ok(Self::CoherentRefactor),
            "coherent-migration" => Ok(Self::CoherentMigration),
            "coherent-spec-impl" => Ok(Self::CoherentSpecImpl),
            "coherent-contract" => Ok(Self::CoherentContract),
            "coherent-test-cover" => Ok(Self::CoherentTestCover),
            "coherent-dep-upgrade" => Ok(Self::CoherentDepUpgrade),
            other => Err(BgroundError::UnknownClaimType(other.to_owned())),
        }
    }
}
