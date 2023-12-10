use crate::check::Check;
use crate::outcome::Outcome;
use crate::semver::Version;
use crate::toolchain::ToolchainSpec;
use crate::TResult;
use std::collections::HashSet;

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
    fn check(&self, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        let v = toolchain.version();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Outcome::new_success(ToolchainSpec::new(
                v.clone(),
                self.target,
            )))
        } else {
            Ok(Outcome::new_failure(
                ToolchainSpec::new(v.clone(), self.target),
                "f".to_string(),
            ))
        }
    }
}
