use crate::{BgroundError, ClaimType};
use std::{collections::BTreeMap, fmt, str::FromStr};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ParsedClaim {
    pub claim_type: ClaimType,
    pub target: String,
    pub assertion: String,
}

impl fmt::Display for ParsedClaim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}",
            self.claim_type, self.target, self.assertion
        )
    }
}

pub struct ClaimString;

impl ClaimString {
    pub fn parse(value: &str) -> Result<ParsedClaim, BgroundError> {
        let mut parts = value.splitn(3, ':');
        let claim_type = required_part(parts.next(), "claim type", value)?;
        let target = required_part(parts.next(), "target", value)?;
        let assertion = required_part(parts.next(), "assertion", value)?;

        Ok(ParsedClaim {
            claim_type: ClaimType::from_str(claim_type)?,
            target: target.to_owned(),
            assertion: assertion.to_owned(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct EvidenceMap(BTreeMap<String, String>);

impl EvidenceMap {
    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self, BgroundError> {
        let mut entries = BTreeMap::new();

        for (key, value) in pairs {
            if key.trim().is_empty() {
                return Err(BgroundError::EvidenceMapInvalid(key));
            }

            entries.insert(key, value);
        }

        Ok(Self(entries))
    }

    pub fn parse_entries(entries: impl IntoIterator<Item = String>) -> Result<Self, BgroundError> {
        Self::from_pairs(
            entries
                .into_iter()
                .map(parse_evidence_entry)
                .collect::<Result<Vec<_>, _>>()?,
        )
    }

    pub fn into_inner(self) -> BTreeMap<String, String> {
        self.0
    }

    pub fn as_inner(&self) -> &BTreeMap<String, String> {
        &self.0
    }
}

fn required_part<'a>(
    part: Option<&'a str>,
    label: &str,
    source: &str,
) -> Result<&'a str, BgroundError> {
    let value = part.ok_or_else(|| BgroundError::ClaimStringMalformed(source.to_owned()))?;

    if value.is_empty() {
        return Err(BgroundError::ClaimStringMalformed(label.to_owned()));
    }

    Ok(value)
}

fn parse_evidence_entry(entry: String) -> Result<(String, String), BgroundError> {
    let (key, value) = entry
        .split_once('=')
        .ok_or_else(|| BgroundError::EvidenceMapInvalid(entry.clone()))?;

    if key.trim().is_empty() {
        return Err(BgroundError::EvidenceMapInvalid(entry));
    }

    Ok((key.to_owned(), value.to_owned()))
}
