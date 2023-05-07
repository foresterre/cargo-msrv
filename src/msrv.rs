use crate::rust_release::RustRelease;
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
    pub fn toolchain(msrv: &RustRelease) -> Self {
        let toolchain = msrv.as_toolchain_spec().to_owned();

        Self::Toolchain { toolchain }
    }

    pub fn from_option(msrv: Option<&RustRelease>) -> Self {
        msrv.map_or(
            MinimumSupportedRustVersion::NoCompatibleToolchain,
            |release| MinimumSupportedRustVersion::toolchain(release),
        )
    }
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
