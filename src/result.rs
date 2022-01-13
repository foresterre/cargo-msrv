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
        // TODO: Optioon or Vec?
        // for linear, option would be more appropriate,
        // for bisect, vec, since we don't necessarily have the last version;
        //      idea: use vec in the test_against_releases and
        //          if nth_release >= 2 && nth_release - 1 was tested, then return Some(Outcome of nth - 1),
        //          else None
        // failures: Option<Vec<Outcome>>,
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
