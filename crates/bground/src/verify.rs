use crate::{BgroundError, ClaimString, EvidenceMap, EvidenceState, ParsedClaim, VerifyArgs};
use bsuite_core::ExitCode;

const PLACEHOLDER_HEADER: &str = "[bground placeholder directive - pre-corpus output]";
const ACTION_PREFIX: &str = "ACTION: This invocation reached bground at the pre-corpus phase.";
const ACTION_SUBSTITUTION: &str =
    "A real evolved directive would name the specific evidence pattern";
const ACTION_TOOLING: &str = "requires and steer the calling LLM toward gathering it (via Read / Bash / WebFetch), or toward retracting the claim if Ungrounded.";
const FOOTER: &str = "Re-invoke after the corpus-backed release lands. Do not treat this placeholder as ground truth. Exit code carries the verdict-class signal.";

pub fn validate_args(args: &VerifyArgs) -> Result<(), BgroundError> {
    parse_invocation(args)?;

    Ok(())
}

pub fn run(args: VerifyArgs) -> Result<std::process::ExitCode, BgroundError> {
    let invocation = parse_invocation(&args)?;
    let evidence_state = EvidenceState::Ungrounded;
    let directive = placeholder_directive(&invocation.claim, evidence_state);

    println!("{directive}");

    Ok(process_exit_code(ExitCode::from(evidence_state)))
}

pub fn placeholder_directive(claim: &ParsedClaim, evidence_state: EvidenceState) -> String {
    format!(
        "{}\nParsed claim: {}. Routing key: ClaimType::{}. Evidence-state: {}.\n{} {} <{}> {}\n{}",
        PLACEHOLDER_HEADER,
        claim,
        claim.claim_type.variant_name(),
        evidence_state_label(evidence_state),
        ACTION_PREFIX,
        ACTION_SUBSTITUTION,
        claim.claim_type.variant_name(),
        ACTION_TOOLING,
        FOOTER,
    )
}

struct VerifyInvocation {
    claim: ParsedClaim,
}

fn parse_invocation(args: &VerifyArgs) -> Result<VerifyInvocation, BgroundError> {
    let claim = ClaimString::parse(&args.claim)?;
    EvidenceMap::from_pairs(args.evidence.clone())?;

    Ok(VerifyInvocation { claim })
}

fn process_exit_code(exit_code: ExitCode) -> std::process::ExitCode {
    std::process::ExitCode::from(exit_code.as_i32() as u8)
}

fn evidence_state_label(evidence_state: EvidenceState) -> &'static str {
    match evidence_state {
        EvidenceState::Grounded => "Grounded",
        EvidenceState::Ungrounded => "Ungrounded",
        EvidenceState::Malformed => "Malformed",
    }
}
