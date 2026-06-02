use crate::{BgroundError, ClaimString, VerifyArgs};

pub fn validate_args(args: &VerifyArgs) -> Result<(), BgroundError> {
    ClaimString::parse(&args.claim)?;

    Ok(())
}

pub fn run(_args: VerifyArgs) -> Result<std::process::ExitCode, BgroundError> {
    // Deferred for the bsuite-core package runtime prompt resolution wiring.
    unimplemented!("not yet implemented")
}
