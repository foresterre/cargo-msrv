//! The outcome of a single toolchain [`check`] run.
//!
//! [`check`]: crate::compatibility::IsCompatible

use crate::rust::Toolchain;
use rust_releases::semver;

#[derive(Clone, Debug)]
pub enum Compatibility {
    Compatible(Compatible),
    Incompatible(Incompatible),
}

impl Compatibility {
    pub fn new_success(toolchain_spec: Toolchain) -> Self {
        Self::Compatible(Compatible { toolchain_spec })
    }

    pub fn new_failure(toolchain_spec: Toolchain, error_message: String) -> Self {
        Self::Incompatible(Incompatible {
            toolchain_spec,
            error_message,
        })
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Compatible { .. } => true,
            Self::Incompatible { .. } => false,
        }
    }

    pub fn version(&self) -> &semver::Version {
        match self {
            Self::Compatible(outcome) => outcome.toolchain_spec.version(),
            Self::Incompatible(outcome) => outcome.toolchain_spec.version(),
        }
    }

    pub fn toolchain_spec(&self) -> &Toolchain {
        match self {
            Self::Compatible(outcome) => &outcome.toolchain_spec,
            Self::Incompatible(outcome) => &outcome.toolchain_spec,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Compatible {
    pub(crate) toolchain_spec: Toolchain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Incompatible {
    pub(crate) toolchain_spec: Toolchain,
    pub(crate) error_message: String,
}

#[cfg(test)]
mod tests {
    use crate::rust::Toolchain;
    use crate::Compatibility;
    use rust_releases::semver;

    #[test]
    fn success_outcome() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        let outcome = Compatibility::new_success(toolchain.clone());

        assert!(outcome.is_success());
        assert_eq!(outcome.version(), &semver::Version::new(1, 2, 3));
        assert_eq!(outcome.toolchain_spec(), &toolchain);
    }

    #[test]
    fn failure_outcome() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        let outcome = Compatibility::new_failure(toolchain.clone(), "msg".to_string());

        assert!(!outcome.is_success());
        assert_eq!(outcome.version(), &semver::Version::new(1, 2, 3));
        assert_eq!(outcome.toolchain_spec(), &toolchain);
    }
}
