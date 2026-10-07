use std::convert::TryFrom;
use version_number::{FullVersion, Version};

pub trait FindMatchingVersion {
    /// Find the first version in `available` which matches this version.
    ///
    /// A two-component version matches any version with the same major and minor version, while a
    /// three-component version only matches the exact same version.
    ///
    /// If `available` is ordered from most-recent to least-recent, it will return the highest matching
    /// semver version for two-component versions, and the exact matching version for three-component versions.
    ///
    /// That is, when our list of available versions is `[0.14.1, 0.14.0, 0.13.0]`, if we supply
    /// a two-component version `0.14`, we will get the result `0.14.1`, while if we supply
    /// the three-component `0.14.0`, we would get the result `0.14.0`.
    fn find_matching_version<'s>(
        &self,
        available: &'s [semver::Version],
    ) -> Result<&'s semver::Version, NoVersionMatchesManifestMsrvError>;
}

impl FindMatchingVersion for Version {
    fn find_matching_version<'s>(
        &self,
        available: &'s [semver::Version],
    ) -> Result<&'s semver::Version, NoVersionMatchesManifestMsrvError> {
        available
            .iter()
            .find(|version| {
                FullVersion::try_from(*version).is_ok_and(|version| self.matches(&version))
            })
            .ok_or_else(|| NoVersionMatchesManifestMsrvError {
                requested: self.clone(),
                available: available.to_vec(),
            })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The MSRV requirement ({requested}) did not match any available version, available: [{}]", .available.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "))]
pub struct NoVersionMatchesManifestMsrvError {
    pub requested: Version,
    pub available: Vec<semver::Version>,
}

#[cfg(test)]
mod tests {
    use super::FindMatchingVersion;
    use version_number::Version;
    use yare::parameterized;

    fn available_versions() -> Vec<semver::Version> {
        vec![
            semver::Version::new(2, 56, 0),
            semver::Version::new(1, 56, 0),
            semver::Version::new(1, 55, 0),
            semver::Version::new(1, 54, 2),
            semver::Version::new(1, 54, 1),
            semver::Version::new(1, 0, 0),
        ]
    }

    #[parameterized(
        two_fifty_six = {  Version::new_base_version(2, 56), semver::Version::new(2, 56, 0) },
        one_fifty_six = {  Version::new_base_version(1, 56), semver::Version::new(1, 56, 0) },
        one_fifty_five = {  Version::new_base_version(1, 55), semver::Version::new(1, 55, 0) },
        one_fifty_four = {  Version::new_base_version(1, 54), semver::Version::new(1, 54, 2) },
        one = {  Version::new_base_version(1, 0), semver::Version::new(1, 0, 0) },
    )]
    fn two_components(version: Version, expected: semver::Version) {
        let versions = available_versions();

        let v = version.find_matching_version(&versions).unwrap();

        assert_eq!(v, &expected);
    }

    #[parameterized(
        two_fifty_six = {  Version::new_full_version(2, 56, 0), semver::Version::new(2, 56, 0) },
        one_fifty_six = {  Version::new_full_version(1, 56, 0), semver::Version::new(1, 56, 0) },
        one_fifty_five = {  Version::new_full_version(1, 55, 0), semver::Version::new(1, 55, 0) },
        one_fifty_four_p2 = {  Version::new_full_version(1, 54, 2), semver::Version::new(1, 54, 2) },
        one_fifty_four_p1 = {  Version::new_full_version(1, 54, 1), semver::Version::new(1, 54, 1) },
        one = {  Version::new_full_version(1, 0, 0), semver::Version::new(1, 0, 0) },
    )]
    fn three_components(version: Version, expected: semver::Version) {
        let versions = available_versions();

        let v = version.find_matching_version(&versions).unwrap();

        assert_eq!(v, &expected);
    }

    #[parameterized(
        three_components = { Version::new_full_version(1, 54, 0) },
        two_components = { Version::new_base_version(1, 57) },
    )]
    fn not_in_index(version: Version) {
        let versions = available_versions();

        assert!(version.find_matching_version(&versions).is_err())
    }

    #[test]
    fn skips_pre_releases() {
        let versions = [
            semver::Version::parse("1.56.1-beta.1").unwrap(),
            semver::Version::new(1, 56, 0),
        ];

        let v = Version::new_base_version(1, 56)
            .find_matching_version(&versions)
            .unwrap();

        assert_eq!(v, &semver::Version::new(1, 56, 0));
    }
}
