use crate::toolchain::ToolchainSpec;
use rust_releases::semver;

/// A `cargo-msrv` Rust release.
///
// FIXME: The next rust-releases version will also contain target information.
#[derive(Clone, Debug)]
pub struct RustRelease {
    release: rust_releases::Release,
    target: String,
}

impl RustRelease {
    pub fn new(release: rust_releases::Release, target: impl Into<String>) -> Self {
        Self {
            release,
            target: target.into(),
        }
    }

    /// Get the [`ToolchainSpec`] for the given Rust release.
    pub fn as_toolchain_spec(&self) -> ToolchainSpec {
        ToolchainSpec::new(self.release.version(), &self.target)
    }
    /// Get the version of the release.
    pub fn version(&self) -> &semver::Version {
        self.release.version()
    }
}
