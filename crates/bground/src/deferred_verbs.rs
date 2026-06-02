use crate::BgroundError;

pub type DeferredResult = Result<std::process::ExitCode, BgroundError>;

pub fn update() -> DeferredResult {
    // Deferred for the bsuite-core package manifest update wiring.
    unimplemented!("not yet implemented")
}

pub fn init() -> DeferredResult {
    // Deferred for the bsuite-core package manifest initialization wiring.
    unimplemented!("not yet implemented")
}

pub fn tail() -> DeferredResult {
    // Deferred for the bsuite-core package transcript reading wiring.
    unimplemented!("not yet implemented")
}

pub fn explain() -> DeferredResult {
    // Deferred for the bsuite-core package directive explanation wiring.
    unimplemented!("not yet implemented")
}
