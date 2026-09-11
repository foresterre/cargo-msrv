use cargo_msrv_types::Toolchain;
use rust_releases::core::{RustRelease, Stable};
use semver::Version;

pub fn to_semver(version: &Stable) -> Version {
    Version::new(
        version.version.major(),
        version.version.minor(),
        version.version.patch(),
    )
}

/// A `cargo-msrv` Rust release.
///
// FIXME: There should be a difference between the available releases, and the requested set of (toolchain, target, component)'s.
//        Only the "Release" part of this struct has been sourced from some Rust release channel, the target and components have been added later
//        but are really items which we want to have; they may not exist. This can be a bit confusing when running cargo msrv with with debug logs on.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolchainCandidate {
    release: RustRelease<Stable>,
    target: &'static str,
    components: &'static [&'static str],
}

impl ToolchainCandidate {
    pub fn new(
        release: RustRelease<Stable>,
        target: &'static str,
        components: &'static [&'static str],
    ) -> Self {
        Self {
            release,
            target,
            components,
        }
    }

    pub fn release(&self) -> &RustRelease<Stable> {
        &self.release
    }

    /// Get the [`Toolchain`] for the given Rust release.
    pub fn to_toolchain_spec(&self) -> Toolchain {
        Toolchain::new(
            to_semver(self.release.version()),
            self.target,
            self.components,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::release::{ToolchainCandidate, to_semver};
    use cargo_msrv_types::Toolchain;
    use rust_releases::core::{RustRelease, Stable};
    use semver::Version;

    #[test]
    fn spec() {
        let candidate =
            ToolchainCandidate::new(RustRelease::new(Stable::new(1, 2, 3), None, []), "x", &[]);
        let spec = candidate.to_toolchain_spec();

        let expected = Toolchain::new(Version::new(1, 2, 3), "x", &[]);
        assert_eq!(spec, expected);
    }

    #[test]
    fn stable_to_semver() {
        assert_eq!(to_semver(&Stable::new(1, 2, 3)), Version::new(1, 2, 3));
    }
}
