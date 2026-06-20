use crate::{
    BgroundError, ClaimString, EvidenceMap, ParsedClaim, VerifyArgs, corpus_index::ClaimCorpusIndex,
};
use bsuite_core::{ExitCode, HostContext, prompt_resolver::DirectiveString};

pub fn run(
    args: &VerifyArgs,
    corpus: &ClaimCorpusIndex,
    _host_context: HostContext,
) -> Result<(DirectiveString, ExitCode), BgroundError> {
    let claim = parse_claim(args)?;
    EvidenceMap::from_pairs(args.evidence.clone())?;

    let directive = corpus.resolve(claim.claim_type).clone();

    Ok((directive, ExitCode::Finding))
}

fn parse_claim(args: &VerifyArgs) -> Result<ParsedClaim, BgroundError> {
    ClaimString::parse(&args.claim)
}
