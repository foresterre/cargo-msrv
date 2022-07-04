use crate::toolchain::OwnedToolchainSpec;
use rust_releases::semver;

#[derive(Clone, Debug)]
pub enum Outcome {
    Success(SuccessOutcome),
    Failure(FailureOutcome),
}

impl Outcome {
    pub fn new_success(toolchain_spec: OwnedToolchainSpec) -> Self {
        Self::Success(SuccessOutcome { toolchain_spec })
    }

    pub fn new_failure(toolchain_spec: OwnedToolchainSpec, error_message: String) -> Self {
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

    pub fn toolchain_spec(&self) -> &OwnedToolchainSpec {
        match self {
            Self::Success(outcome) => &outcome.toolchain_spec,
            Self::Failure(outcome) => &outcome.toolchain_spec,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessOutcome {
    pub(crate) toolchain_spec: OwnedToolchainSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureOutcome {
    pub(crate) toolchain_spec: OwnedToolchainSpec,
    pub(crate) error_message: String,
}
