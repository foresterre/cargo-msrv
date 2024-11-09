// Copied from src/testing.rs for integration and end-to-end testing
// To do: Merge them back together in a testing dev-dep crate:
// * Requires: traits for Check, Output etc. to be separated to a library crate as
//      well.

use std::collections::HashSet;

use cargo_msrv::compatibility::IsCompatible;
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::rust::Toolchain;
use cargo_msrv::Compatibility;
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

impl IsCompatible for TestRunner {
    fn is_compatible(&self, toolchain: &Toolchain) -> Result<Compatibility, CargoMSRVError> {
        let version = toolchain.version();
        let components = toolchain.components();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Compatibility::new_success(Toolchain::new(
                version.clone(),
                self.target,
                components,
            )))
        } else {
            Ok(Compatibility::new_failure(
                Toolchain::new(version.clone(), self.target, components),
                "f".to_string(),
            ))
        }
    }
}
