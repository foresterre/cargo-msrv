use rust_releases::semver;

use crate::check::Outcome;

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
    NoCompatibleToolchains,
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
        let version = outcome.version().clone();
        let toolchain = outcome.toolchain().to_string();

        if outcome.is_success() {
            MinimalCompatibility::CapableToolchain { version, toolchain }
        } else {
            MinimalCompatibility::NoCompatibleToolchains
        }
    }
}
