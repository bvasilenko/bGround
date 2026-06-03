use bground::{EvidenceState, ParsedClaim};
pub fn placeholder_header() -> &'static str {
    "[bground placeholder directive - pre-corpus output]"
}

pub fn assert_public_voice_safe_runtime_output(output: &str) {
    for prohibited in prohibited_runtime_phrases() {
        assert!(
            !output.contains(&prohibited),
            "runtime output contains prohibited public-voice phrase: {prohibited}"
        );
    }
}

pub fn assert_placeholder_directive(
    output: &str,
    claim: &ParsedClaim,
    evidence_state: EvidenceState,
) {
    assert_public_voice_safe_runtime_output(output);
    assert!(output.contains(placeholder_header()));
    assert!(output.contains(&format!("Parsed claim: {claim}.")));
    assert!(output.contains(&format!(
        "Routing key: ClaimType::{}.",
        claim.claim_type.variant_name()
    )));
    assert!(output.contains(&format!(
        "Evidence-state: {}.",
        evidence_state_label(evidence_state)
    )));
    assert!(output.contains("ACTION: This invocation reached bground at the pre-corpus phase."));
    assert!(output.contains(&format!(
        "A real evolved directive would name the specific evidence pattern <{}> requires",
        claim.claim_type.variant_name()
    )));
    assert!(output.contains("Exit code carries the verdict-class signal."));
}

fn prohibited_runtime_phrases() -> Vec<String> {
    vec![["0.1.0", "-skeleton"].concat(), ["v0.2", "+"].concat()]
}

fn evidence_state_label(evidence_state: EvidenceState) -> &'static str {
    match evidence_state {
        EvidenceState::Grounded => "Grounded",
        EvidenceState::Ungrounded => "Ungrounded",
        EvidenceState::Malformed => "Malformed",
    }
}
