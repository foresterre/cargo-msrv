use crate::toolchain::ToolchainSpec;

/// A `cargo-msrv` Rust release.
///
// FIXME: The next rust-releases version will also contain target information.
#[derive(Clone, Debug, PartialEq)]
pub struct RustRelease {
    release: rust_releases::Release,
    target: &'static str,
}

impl RustRelease {
    pub fn new(release: rust_releases::Release, target: &'static str) -> Self {
        Self { release, target }
    }

    /// Get the [`ToolchainSpec`] for the given Rust release.
    pub fn to_toolchain_spec(&self) -> ToolchainSpec {
        let version = self.release.version();
        ToolchainSpec::new(version.clone(), self.target)
    }
}

#[cfg(test)]
mod tests {
    use crate::rust_release::RustRelease;
    use crate::toolchain::ToolchainSpec;
    use rust_releases::semver;

    #[test]
    fn spec() {
        let version = semver::Version::new(1, 2, 3);
        let rust_release =
            RustRelease::new(rust_releases::Release::new_stable(version.clone()), "x");
        let spec = rust_release.to_toolchain_spec();

        let expected = ToolchainSpec::new(version, "x");
        assert_eq!(spec, expected);
    }
}
