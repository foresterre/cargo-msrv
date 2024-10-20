use crate::rust::Toolchain;

/// A `cargo-msrv` Rust release.
///
// FIXME: The next rust-releases version will also contain target information.
// FIXME: There should be a difference between the available releases, and the requested set of (toolchain, target, component)'s.
//        Only the "Release" part of this struct has been sourced from some Rust release channel, the target and components have been added later
//        but are really items which we want to have; they may not exist. This can be a bit confusing when running cargo msrv with with debug logs on.
#[derive(Clone, Debug, PartialEq)]
pub struct RustRelease {
    release: rust_releases::Release,
    target: &'static str,
    components: &'static [&'static str],
}

impl RustRelease {
    pub fn new(
        release: rust_releases::Release,
        target: &'static str,
        components: &'static [&'static str],
    ) -> Self {
        Self {
            release,
            target,
            components,
        }
    }

    /// Get the [`Toolchain`] for the given Rust release.
    pub fn to_toolchain_spec(&self) -> Toolchain {
        let version = self.release.version();
        Toolchain::new(version.clone(), self.target, self.components)
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::RustRelease;
    use crate::rust::Toolchain;
    use rust_releases::semver;

    #[test]
    fn spec() {
        let version = semver::Version::new(1, 2, 3);
        let rust_release = RustRelease::new(
            rust_releases::Release::new_stable(version.clone()),
            "x",
            &[],
        );
        let spec = rust_release.to_toolchain_spec();

        let expected = Toolchain::new(version, "x", &[]);
        assert_eq!(spec, expected);
    }
}
