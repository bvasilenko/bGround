mod common;

use bground::{ClaimString, ClaimType, EvidenceState, verify};
use bsuite_core::ExitCode;

#[test]
fn placeholder_directive_names_claim_route_evidence_state_action_and_exit_signal() {
    let claim = ClaimString::parse("file-exists:README.md:README exists").unwrap();
    let directive = verify::placeholder_directive(&claim, EvidenceState::Ungrounded);

    common::assert_placeholder_directive(&directive, &claim, EvidenceState::Ungrounded);
    assert_eq!(ExitCode::from(EvidenceState::Ungrounded), ExitCode::Finding);
}

#[test]
fn placeholder_directive_contract_holds_for_every_route_and_evidence_state() {
    for claim_type in ClaimType::ALL {
        let claim = ClaimString::parse(&format!("{claim_type}:target:assertion")).unwrap();

        for evidence_state in EvidenceState::ALL {
            let directive = verify::placeholder_directive(&claim, evidence_state);

            common::assert_placeholder_directive(&directive, &claim, evidence_state);
            assert_eq!(directive.matches(common::placeholder_header()).count(), 1);
            assert_eq!(directive.matches("ACTION:").count(), 1);
            assert_eq!(
                directive
                    .matches("Exit code carries the verdict-class signal.")
                    .count(),
                1
            );
        }
    }
}
