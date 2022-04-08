// Copied from src/testing.rs for integration and end-to-end testing
// To do: Merge them back together in a testing dev-dep crate:
// * Requires: traits for Check, Output etc. to be separated to a library crate as
//      well.

use std::collections::HashSet;

use cargo_msrv::check::Check;
use cargo_msrv::config::Config;
use cargo_msrv::errors::TResult;
use cargo_msrv::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use cargo_msrv::Outcome;
use rust_releases::semver;
use rust_releases::semver::Version;

pub struct TestRunner {
    accept_versions: HashSet<semver::Version>,
}

impl TestRunner {
    pub fn with_ok<'v, T: IntoIterator<Item = &'v Version>>(iter: T) -> Self {
        Self {
            accept_versions: iter.into_iter().cloned().collect(),
        }
    }
}

impl Check for TestRunner {
    fn check(&self, config: &Config, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        let v = toolchain.version();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Outcome::new_success(OwnedToolchainSpec::new(
                v,
                config.target(),
            )))
        } else {
            Ok(Outcome::new_failure(
                OwnedToolchainSpec::new(v, config.target()),
                "f".to_string(),
            ))
        }
    }
}
