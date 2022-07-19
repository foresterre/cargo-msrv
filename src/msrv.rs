use crate::toolchain::OwnedToolchainSpec;

/// An enum to represent the minimal compatibility
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MinimumSupportedRustVersion {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    Toolchain {
        // toolchain
        toolchain: OwnedToolchainSpec,
    },
    /// Compatibility is none, if the check on the last available toolchain fails
    NoCompatibleToolchain,
}

impl MinimumSupportedRustVersion {
    #[cfg(test)]
    pub fn unwrap_version(&self) -> rust_releases::semver::Version {
        if let Self::Toolchain { toolchain, .. } = self {
            return toolchain.version().clone();
        }

        panic!("Unable to unwrap MinimalCompatibility (CapableToolchain::version)")
    }
}
