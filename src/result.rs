use rust_releases::semver;

use crate::outcome::Outcome;

/// An enum to represent the minimal compatibility
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MinimalCompatibility {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    CapableToolchain {
        // toolchain specifier
        toolchain: String,
        // checked Rust version
        version: semver::Version,
    },
    /// Compatibility is none, if the check on the last available toolchain fails
    NoCompatibleToolchains { reason: Option<String> },
}

impl MinimalCompatibility {
    pub fn unwrap_version(&self) -> semver::Version {
        if let Self::CapableToolchain { version, .. } = self {
            return version.clone();
        }

        panic!("Unable to unwrap MinimalCompatibility (CapableToolchain::version)")
    }
}

impl From<Outcome> for MinimalCompatibility {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Success { toolchain } => MinimalCompatibility::CapableToolchain {
                version: toolchain.version().clone(),
                toolchain: toolchain.spec().to_string(),
            },
            Outcome::Failure {
                toolchain: _,
                error_reason,
            } => MinimalCompatibility::NoCompatibleToolchains {
                reason: error_reason,
            },
        }
    }
}
