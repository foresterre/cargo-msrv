// Copied from src/testing.rs for integration and end-to-end testing
// To do: Merge them back together in a testing dev-dep crate:
// * Requires: traits for Check, Output etc. to be separated to a library crate as
//      well.

use std::collections::HashSet;

use cargo_msrv::check::Check;
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::rust::Toolchain;
use cargo_msrv::Outcome;
use rust_releases::semver::Version;

pub struct TestRunner {
    accept_versions: HashSet<Version>,
    target: &'static str,
}

impl TestRunner {
    pub fn with_ok<'v, T: IntoIterator<Item = &'v Version>>(target: &'static str, iter: T) -> Self {
        Self {
            accept_versions: iter.into_iter().cloned().collect(),
            target,
        }
    }
}

impl Check for TestRunner {
    fn check(&self, toolchain: &Toolchain) -> Result<Outcome, CargoMSRVError> {
        let version = toolchain.version();
        let components = toolchain.components();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Outcome::new_success(Toolchain::new(
                version.clone(),
                self.target,
                components,
            )))
        } else {
            Ok(Outcome::new_failure(
                Toolchain::new(version.clone(), self.target, components),
                "f".to_string(),
            ))
        }
    }
}
