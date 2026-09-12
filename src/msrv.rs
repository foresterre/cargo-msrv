use crate::rust::Toolchain;

/// An enum to represent the minimal compatibility
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MinimumSupportedRustVersion {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    Toolchain {
        // toolchain
        toolchain: Toolchain,
    },
    /// Compatibility is none, if the check on the last available toolchain fails
    NoCompatibleToolchain,
}

impl MinimumSupportedRustVersion {
    pub fn toolchain(toolchain: Toolchain) -> Self {
        Self::Toolchain { toolchain }
    }

    pub fn from_option(msrv: Option<Toolchain>) -> Self {
        msrv.map_or(
            MinimumSupportedRustVersion::NoCompatibleToolchain,
            MinimumSupportedRustVersion::toolchain,
        )
    }
}

impl MinimumSupportedRustVersion {
    #[cfg(test)]
    pub fn unwrap_version(&self) -> semver::Version {
        if let Self::Toolchain { toolchain, .. } = self {
            return toolchain.version().clone();
        }

        panic!("Unable to unwrap MinimalCompatibility (CapableToolchain::version)")
    }
}

#[cfg(test)]
mod tests {
    use crate::msrv::MinimumSupportedRustVersion;
    use crate::rust::Toolchain;

    #[test]
    fn accept() {
        let version = semver::Version::new(1, 2, 3);
        let msrv =
            MinimumSupportedRustVersion::toolchain(Toolchain::new(version.clone(), "x", &[]));

        assert!(matches!(
            msrv,
            MinimumSupportedRustVersion::Toolchain { toolchain } if toolchain.version() == &version && toolchain.target() == "x"));
    }

    #[test]
    fn accept_from_option() {
        let version = semver::Version::new(1, 2, 3);
        let msrv = MinimumSupportedRustVersion::from_option(Some(Toolchain::new(
            version.clone(),
            "x",
            &[],
        )));

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
