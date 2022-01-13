use crate::toolchain::OwnedToolchainSpec;
use rust_releases::semver;

#[derive(Clone, Debug)]
pub enum Outcome {
    Success {
        toolchain: OwnedToolchainSpec,
    },
    Failure {
        toolchain: OwnedToolchainSpec,
        // output of the check failure
        error_reason: Option<String>,
    },
}

impl Outcome {
    pub fn new_success(toolchain: OwnedToolchainSpec) -> Self {
        Self::Success { toolchain }
    }

    pub fn new_failure(toolchain: OwnedToolchainSpec, error_reason: String) -> Self {
        Self::Failure {
            toolchain,
            error_reason: Some(error_reason),
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Success { .. } => true,
            Self::Failure { .. } => false,
        }
    }

    pub fn version(&self) -> &semver::Version {
        match self {
            Self::Success { toolchain } => toolchain.version(),
            Self::Failure { toolchain, .. } => toolchain.version(),
        }
    }

    pub fn toolchain_spec(&self) -> &str {
        match self {
            Self::Success { toolchain } => toolchain.spec(),
            Self::Failure { toolchain, .. } => toolchain.spec(),
        }
    }
}
