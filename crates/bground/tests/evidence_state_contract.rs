use bground::EvidenceState;
use bsuite_core::ExitCode;
use proptest::prelude::*;
use std::{collections::BTreeSet, str::FromStr};

const EXPECTED_STATES: [(EvidenceState, &str, ExitCode); 3] = [
    (EvidenceState::Grounded, "grounded", ExitCode::Success),
    (EvidenceState::Ungrounded, "ungrounded", ExitCode::Finding),
    (EvidenceState::Malformed, "malformed", ExitCode::Usage),
];

fn evidence_state_strategy() -> impl Strategy<Value = EvidenceState> {
    prop::sample::select(&EvidenceState::ALL)
}

proptest! {
    #[test]
    fn supported_evidence_states_round_trip(evidence_state in evidence_state_strategy()) {
        let stable_name = evidence_state.to_string();
        prop_assert_eq!(EvidenceState::from_str(&stable_name), Ok(evidence_state));
        prop_assert_eq!(EvidenceState::from_str(&stable_name)?.stable_name(), stable_name);
    }
}

#[test]
fn evidence_state_set_is_closed_ordered_unique_and_mapped_to_exit_codes() {
    let actual_states =
        EvidenceState::ALL.map(|state| (state, state.stable_name(), ExitCode::from(state)));
    let unique_names = actual_states
        .iter()
        .map(|(_, name, _)| *name)
        .collect::<BTreeSet<_>>();

    assert_eq!(actual_states, EXPECTED_STATES);
    assert_eq!(unique_names.len(), EXPECTED_STATES.len());
}

#[test]
fn supported_evidence_state_names_use_lowercase_kebab_case() {
    for evidence_state in EvidenceState::ALL {
        let name = evidence_state.stable_name();

        assert!(!name.is_empty());
        assert!(name.is_ascii());
        assert!(
            name.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        );
    }
}

#[test]
fn unsupported_evidence_states_are_rejected_without_normalization() {
    for value in [
        "",
        " ",
        "ground",
        "Grounded",
        "GROUNDED",
        "grounded ",
        " grounded",
        "unknown",
    ] {
        assert!(
            EvidenceState::from_str(value).is_err(),
            "accepted {value:?}"
        );
    }
}
