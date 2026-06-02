use bground::{VerifyArgs, deferred_verbs, verify};
use std::{any::Any, collections::BTreeSet, panic, path::PathBuf, sync::Mutex};

type DeferredCall = fn();

static PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());

struct DeferredVerbCase {
    name: &'static str,
    run: DeferredCall,
}

fn deferred_verb_cases() -> [DeferredVerbCase; 5] {
    [
        DeferredVerbCase {
            name: "verify",
            run: call_verify,
        },
        DeferredVerbCase {
            name: "update",
            run: call_update,
        },
        DeferredVerbCase {
            name: "init",
            run: call_init,
        },
        DeferredVerbCase {
            name: "tail",
            run: call_tail,
        },
        DeferredVerbCase {
            name: "explain",
            run: call_explain,
        },
    ]
}

fn call_verify() {
    let _ = verify::run(verify_args());
}

fn call_update() {
    let _ = deferred_verbs::update();
}

fn call_init() {
    let _ = deferred_verbs::init();
}

fn call_tail() {
    let _ = deferred_verbs::tail();
}

fn call_explain() {
    let _ = deferred_verbs::explain();
}

fn verify_args() -> VerifyArgs {
    VerifyArgs {
        claim: "file-exists:README.md:README exists".to_owned(),
        evidence: Vec::new(),
        manifest: Option::<PathBuf>::None,
        json: false,
        quiet: false,
        reason: Option::<String>::None,
    }
}

fn assert_not_yet_implemented(call: DeferredCall) {
    let _hook_guard = PANIC_HOOK_LOCK.lock().expect("panic hook lock poisoned");
    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(|_| {}));
    let panic_result = panic::catch_unwind(call);
    panic::set_hook(default_hook);

    let panic_payload = panic_result.expect_err("deferred callable returned");
    let message = panic_message(panic_payload.as_ref());

    assert!(
        message.contains("not yet implemented"),
        "unexpected panic message: {message}"
    );
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_owned();
    }

    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "<non-string panic>".to_owned()
}

#[test]
fn every_deferred_verb_has_a_unique_contract_case() {
    let cases = deferred_verb_cases();
    let unique_names = cases.iter().map(|case| case.name).collect::<BTreeSet<_>>();

    assert_eq!(cases.len(), 5);
    assert_eq!(unique_names.len(), cases.len());
}

#[test]
fn every_deferred_verb_is_an_explicit_placeholder() {
    for case in deferred_verb_cases() {
        assert_not_yet_implemented(case.run);
    }
}
