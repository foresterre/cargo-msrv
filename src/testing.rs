use crate::check::Check;
use crate::outcome::Outcome;
use crate::semver::Version;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Action, Config, Output, ProgressAction, TResult};
use rust_releases::semver;
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

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
