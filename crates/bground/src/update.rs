use crate::BgroundError;
use bsuite_core::{SignedManifestUpdater, UpdateChannel, UpdateOutcome, Updater};
use semver::Version;
use std::path::Path;

const UPDATE_BASE_URL_ENV: &str = "BSUITE_UPDATE_BASE_URL";
const UPDATE_BASE_URL_PLACEHOLDER: &str = "https://updates.example.invalid/bground/v1";

pub fn run(install_dir: &Path) -> Result<(), BgroundError> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION is always valid semver");

    let base_url = std::env::var(UPDATE_BASE_URL_ENV)
        .unwrap_or_else(|_| UPDATE_BASE_URL_PLACEHOLDER.to_owned());

    let channel = UpdateChannel::new(base_url);

    let updater = SignedManifestUpdater::new().map_err(BgroundError::Core)?;

    let outcome = updater
        .check(&current_version, &channel)
        .map_err(BgroundError::Core)?;

    match &outcome {
        UpdateOutcome::UpToDate => {
            eprintln!("already at the latest version");
        }
        UpdateOutcome::UpgradeAvailable { manifest, .. } => {
            eprintln!("upgrading to version {}", manifest.version);
            updater
                .apply(&outcome, install_dir)
                .map_err(BgroundError::Core)?;
            eprintln!("upgrade complete");
        }
    }

    Ok(())
}
