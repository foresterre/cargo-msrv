use std::cmp::Ordering;
use std::fmt;
use version_number::{BaseVersion, FullVersion, Version};

/// A type to represent the Minimal Supported Rust Version, also known as the
/// MSRV.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct MSRV {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    version: Version,
}

impl MSRV {
    pub const fn new(msrv: Version) -> Self {
        Self { version: msrv }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn short_version(&self) -> BaseVersion {
        BaseVersion::new(self.version.major(), self.version.minor())
    }
}

impl From<Version> for MSRV {
    fn from(version: Version) -> Self {
        Self::new(version)
    }
}

impl From<BaseVersion> for MSRV {
    fn from(version: BaseVersion) -> Self {
        Self::new(Version::from(version))
    }
}

impl From<FullVersion> for MSRV {
    fn from(version: FullVersion) -> Self {
        Self::new(Version::from(version))
    }
}

impl PartialOrd for MSRV {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MSRV {
    fn cmp(&self, other: &Self) -> Ordering {
        let is_full = |msrv: &Self| msrv.version.patch().is_some();

        self.version
            .to_full_version_lossy()
            .cmp(&other.version.to_full_version_lossy())
            .then_with(|| is_full(self).cmp(&is_full(other)))
    }
}

impl fmt::Display for MSRV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.version))
    }
}

#[cfg(test)]
mod tests {
    use crate::msrv::MSRV;
    use version_number::{BaseVersion, FullVersion, Version};

    #[test]
    fn version_method() {
        let version = Version::new_full_version(1, 2, 3);
        let msrv = MSRV::new(version.clone());

        assert_eq!(msrv.version(), &version);
    }

    #[test]
    fn version_str() {
        let msrv = MSRV::new(Version::new_full_version(1, 2, 3));

        assert_eq!(msrv.to_string(), "1.2.3".to_string());
    }

    #[test]
    fn base_version_str() {
        let msrv = MSRV::new(Version::new_base_version(1, 2));

        assert_eq!(msrv.to_string(), "1.2".to_string());
    }

    #[test]
    fn short_version() {
        let msrv = MSRV::new(Version::new_full_version(1, 2, 3));

        assert_eq!(msrv.short_version(), BaseVersion::new(1, 2));
    }

    #[test]
    fn from_base_and_full() {
        assert_eq!(
            MSRV::from(BaseVersion::new(1, 2)),
            MSRV::new(Version::new_base_version(1, 2))
        );
        assert_eq!(
            MSRV::from(FullVersion::new(1, 2, 3)),
            MSRV::new(Version::new_full_version(1, 2, 3))
        );
    }

    #[yare::parameterized(
        lower_minor = { Version::new_full_version(1, 55, 9), Version::new_full_version(1, 56, 0) },
        lower_patch = { Version::new_full_version(1, 56, 0), Version::new_full_version(1, 56, 1) },
        base_before_full = { Version::new_base_version(1, 56), Version::new_full_version(1, 56, 0) },
        base_before_higher_full = { Version::new_base_version(1, 56), Version::new_full_version(1, 56, 1) },
        full_before_higher_base = { Version::new_full_version(1, 56, 1), Version::new_base_version(1, 57) },
    )]
    fn ordering(lower: Version, higher: Version) {
        let lower = MSRV::new(lower);
        let higher = MSRV::new(higher);

        assert!(lower < higher);
        assert!(higher > lower);
    }
}
