use crate::compatibility::IsCompatible;
use crate::outcome::Compatibility;
use crate::rust::Toolchain;
use crate::semver::Version;
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

impl IsCompatible for TestRunner {
    fn is_compatible(&self, toolchain: &Toolchain) -> TResult<Compatibility> {
        let v = toolchain.version();

        if self.accept_versions.contains(toolchain.version()) {
            Ok(Compatibility::new_success(Toolchain::new(
                v.clone(),
                self.target,
                &[],
            )))
        } else {
            Ok(Compatibility::new_failure(
                Toolchain::new(v.clone(), self.target, &[]),
                "f".to_string(),
            ))
        }
    }
}
