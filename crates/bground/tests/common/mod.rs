pub fn assert_public_voice_safe_runtime_output(output: &str) {
    for prohibited in prohibited_runtime_phrases() {
        assert!(
            !output.contains(&prohibited),
            "runtime output contains prohibited public-voice phrase: {prohibited}"
        );
    }
}

pub fn assert_corpus_directive(output: &str) {
    assert_public_voice_safe_runtime_output(output);
    assert!(!output.is_empty(), "corpus directive must be non-empty");
    assert!(
        !output.contains("[bground placeholder directive"),
        "corpus-backed output must not contain the placeholder header"
    );
}

fn prohibited_runtime_phrases() -> Vec<String> {
    vec![["0.1.0", "-skeleton"].concat(), ["v0.2", "+"].concat()]
}
