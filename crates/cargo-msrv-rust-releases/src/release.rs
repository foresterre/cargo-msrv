use rust_releases::core::Stable;
use semver::Version;

pub fn to_semver(version: &Stable) -> Version {
    Version::new(
        version.version.major(),
        version.version.minor(),
        version.version.patch(),
    )
}

#[cfg(test)]
mod tests {
    use crate::release::to_semver;
    use rust_releases::core::Stable;
    use semver::Version;

    #[test]
    fn stable_to_semver() {
        assert_eq!(to_semver(&Stable::new(1, 2, 3)), Version::new(1, 2, 3));
    }
}
