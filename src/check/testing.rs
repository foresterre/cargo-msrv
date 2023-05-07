use crate::check::Check;
use crate::outcome::Outcome;
use crate::semver::Version;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, TResult};
use std::collections::HashSet;

pub struct TestRunner {
    accept_versions: HashSet<Version>,
    settings: Settings,
}

impl TestRunner {
    pub fn with_ok<'v, T: IntoIterator<Item = &'v Version>>(target: String, iter: T) -> Self {
        Self {
            accept_versions: iter.into_iter().cloned().collect(),
            settings: Settings { target },
        }
    }
}

impl Check for TestRunner {
    fn check(&self, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        let v = toolchain.version();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Outcome::new_success(OwnedToolchainSpec::new(
                v,
                &self.settings.target,
            )))
        } else {
            Ok(Outcome::new_failure(
                OwnedToolchainSpec::new(v, &self.settings.target),
                "f".to_string(),
            ))
        }
    }
}

struct Settings {
    target: String,
}
