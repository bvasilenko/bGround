use crate::{BgroundError, ClaimType};
use bsuite_core::{corpus::parse_signed_corpus, prompt_resolver::DirectiveString};
use ed25519_dalek::VerifyingKey;
use serde::Deserialize;
use std::collections::HashMap;

/// Extended per-entry schema that adds the bGround-local `claim_type`
/// discriminator field.  bCore's `CorpusEntry` does not carry `claim_type`
/// and does not use `deny_unknown_fields`, so this field is present in the
/// TOML but invisible to bCore's signature computation.  The mitigating
/// invariant: completeness validation at construction requires all 16
/// `ClaimType` variants present exactly once, and the embedded corpus bytes
/// are compile-time constants so they cannot be tampered at runtime.
#[derive(Deserialize)]
struct ExtendedCorpusFile {
    entries: Vec<ExtendedCorpusEntry>,
}

#[derive(Deserialize)]
struct ExtendedCorpusEntry {
    claim_type: String,
    directive: String,
}

/// Invariant upheld at construction: every `ClaimType` variant maps to
/// exactly one directive.  `resolve` is therefore infallible after
/// construction succeeds.
pub struct ClaimCorpusIndex {
    by_claim_type: HashMap<ClaimType, DirectiveString>,
}

impl std::fmt::Debug for ClaimCorpusIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaimCorpusIndex")
            .field("entry_count", &self.by_claim_type.len())
            .finish()
    }
}

impl ClaimCorpusIndex {
    pub fn from_toml_signed(
        corpus_toml: &str,
        pubkey: &VerifyingKey,
    ) -> Result<Self, BgroundError> {
        parse_signed_corpus(corpus_toml, pubkey).map_err(BgroundError::Core)?;

        let extended: ExtendedCorpusFile =
            toml::from_str(corpus_toml).map_err(|e| BgroundError::CorpusLoad(e.to_string()))?;

        Self::build_index(extended.entries)
    }

    fn build_index(entries: Vec<ExtendedCorpusEntry>) -> Result<Self, BgroundError> {
        let mut by_claim_type: HashMap<ClaimType, DirectiveString> =
            HashMap::with_capacity(ClaimType::ALL.len());

        for entry in entries {
            let claim_type = entry.claim_type.parse::<ClaimType>().map_err(|_| {
                BgroundError::CorpusLoad(format!(
                    "unrecognised claim_type in corpus: {}",
                    entry.claim_type
                ))
            })?;

            if by_claim_type.contains_key(&claim_type) {
                return Err(BgroundError::CorpusLoad(format!(
                    "duplicate claim_type in corpus: {}",
                    entry.claim_type
                )));
            }

            by_claim_type.insert(claim_type, DirectiveString::new(entry.directive));
        }

        for variant in ClaimType::ALL {
            if !by_claim_type.contains_key(&variant) {
                return Err(BgroundError::CorpusLoad(format!(
                    "corpus missing entry for claim_type: {}",
                    variant.stable_name()
                )));
            }
        }

        Ok(Self { by_claim_type })
    }

    /// Panics only if the invariant enforced at construction is somehow
    /// violated, which is impossible through the public API.
    pub fn resolve(&self, claim_type: ClaimType) -> &DirectiveString {
        self.by_claim_type
            .get(&claim_type)
            .expect("ClaimCorpusIndex invariant: every ClaimType variant indexed at construction")
    }
}
