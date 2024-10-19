//! The outcome of a single toolchain [`check`] run.
//!
//! [`check`]: crate::check::Check

use crate::rust::Toolchain;
use rust_releases::semver;

#[derive(Clone, Debug)]
pub enum Outcome {
    Success(SuccessOutcome),
    Failure(FailureOutcome),
}

impl Outcome {
    pub fn new_success(toolchain_spec: Toolchain) -> Self {
        Self::Success(SuccessOutcome { toolchain_spec })
    }

    pub fn new_failure(toolchain_spec: Toolchain, error_message: String) -> Self {
        Self::Failure(FailureOutcome {
            toolchain_spec,
            error_message,
        })
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Success { .. } => true,
            Self::Failure { .. } => false,
        }
    }

    pub fn version(&self) -> &semver::Version {
        match self {
            Self::Success(outcome) => outcome.toolchain_spec.version(),
            Self::Failure(outcome) => outcome.toolchain_spec.version(),
        }
    }

    pub fn toolchain_spec(&self) -> &Toolchain {
        match self {
            Self::Success(outcome) => &outcome.toolchain_spec,
            Self::Failure(outcome) => &outcome.toolchain_spec,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessOutcome {
    pub(crate) toolchain_spec: Toolchain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureOutcome {
    pub(crate) toolchain_spec: Toolchain,
    pub(crate) error_message: String,
}

#[cfg(test)]
mod tests {
    use crate::rust::Toolchain;
    use crate::Outcome;
    use rust_releases::semver;

    #[test]
    fn success_outcome() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        let outcome = Outcome::new_success(toolchain.clone());

        assert!(outcome.is_success());
        assert_eq!(outcome.version(), &semver::Version::new(1, 2, 3));
        assert_eq!(outcome.toolchain_spec(), &toolchain);
    }

    #[test]
    fn failure_outcome() {
        let version = semver::Version::new(1, 2, 3);
        let toolchain = Toolchain::new(version, "x", &[]);

        let outcome = Outcome::new_failure(toolchain.clone(), "msg".to_string());

        assert!(!outcome.is_success());
        assert_eq!(outcome.version(), &semver::Version::new(1, 2, 3));
        assert_eq!(outcome.toolchain_spec(), &toolchain);
    }
}
