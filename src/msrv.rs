use crate::rust_release::RustRelease;
use crate::toolchain::ToolchainSpec;

/// An enum to represent the minimal compatibility
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MinimumSupportedRustVersion {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    Toolchain {
        // toolchain
        toolchain: ToolchainSpec,
    },
    /// Compatibility is none, if the check on the last available toolchain fails
    NoCompatibleToolchain,
}

impl MinimumSupportedRustVersion {
    pub fn toolchain(msrv: &RustRelease) -> Self {
        let toolchain = msrv.to_toolchain_spec().to_owned();

        Self::Toolchain { toolchain }
    }

    pub fn from_option(msrv: Option<&RustRelease>) -> Self {
        msrv.map_or(
            MinimumSupportedRustVersion::NoCompatibleToolchain,
            MinimumSupportedRustVersion::toolchain,
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

#[cfg(test)]
mod tests {
    use crate::msrv::MinimumSupportedRustVersion;
    use crate::rust_release::RustRelease;
    use cargo_metadata::semver;

    #[test]
    fn accept() {
        let version = semver::Version::new(1, 2, 3);
        let rust_release = RustRelease::new(
            rust_releases::Release::new_stable(version.clone()),
            "x",
            &[],
        );
        let msrv = MinimumSupportedRustVersion::toolchain(&rust_release);

        assert!(matches!(
            msrv,
            MinimumSupportedRustVersion::Toolchain { toolchain } if toolchain.version() == &version && toolchain.target() == "x"));
    }

    #[test]
    fn accept_from_option() {
        let version = semver::Version::new(1, 2, 3);
        let rust_release = RustRelease::new(
            rust_releases::Release::new_stable(version.clone()),
            "x",
            &[],
        );
        let msrv = MinimumSupportedRustVersion::from_option(Some(&rust_release));

        assert!(matches!(
            msrv,
            MinimumSupportedRustVersion::Toolchain { toolchain } if toolchain.version() == &version && toolchain.target() == "x"));
    }

    #[test]
    fn reject_from_option() {
        let msrv = MinimumSupportedRustVersion::from_option(None);

        assert!(matches!(
            msrv,
            MinimumSupportedRustVersion::NoCompatibleToolchain
        ));
    }
}
