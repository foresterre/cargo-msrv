use crate::manifest::bare_version;
use crate::semver;
use rust_releases::linear::LatestStableReleases;
use rust_releases::Release;

/// Filter releases based on the given configuration.
pub struct ReleasesFilter<'ctx> {
    include_all_patch_releases: bool,
    minimum_version: Option<&'ctx bare_version::BareVersion>,
    maximum_version: Option<&'ctx bare_version::BareVersion>,
}

impl<'ctx> ReleasesFilter<'ctx> {
    /// Initiate a new filter
    pub fn new(
        include_all_patch_releases: bool,
        minimum_version: Option<&'ctx bare_version::BareVersion>,
        maximum_version: Option<&'ctx bare_version::BareVersion>,
    ) -> Self {
        Self {
            include_all_patch_releases,
            minimum_version,
            maximum_version,
        }
    }

    /// Filter the given slice of releases, based on the options set for the filter.
    pub fn filter(&self, releases: &[Release]) -> Vec<Release> {
        let releases = if self.include_all_patch_releases {
            releases.to_vec()
        } else {
            releases.iter().cloned().latest_stable_releases().collect()
        };

        // Pre-filter the [min-version:max-version] range
        releases
            .into_iter()
            .filter(|release| {
                include_version(
                    release.version(),
                    self.minimum_version,
                    self.maximum_version,
                )
            })
            .collect::<Vec<_>>()
    }
}

fn include_version(
    current: &semver::Version,
    min_version: Option<&bare_version::BareVersion>,
    max_version: Option<&bare_version::BareVersion>,
) -> bool {
    match (min_version, &max_version) {
        (Some(min), Some(max)) => min.is_at_least(current) && max.is_at_most(current),
        (Some(min), None) => min.is_at_least(current),
        (None, Some(max)) => max.is_at_most(current),
        (None, None) => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::bare_version::BareVersion;
    use parameterized::{ide, parameterized};
    use rust_releases::semver::Version;

    use super::*;

    ide!();

    #[test]
    fn max_should_ignore_patch() {
        let current = Version::new(1, 54, 1);
        let max_version = BareVersion::TwoComponents(1, 54);

        assert!(include_version(&current, None, Some(max_version).as_ref()));
    }

    #[test]
    fn max_should_be_strict_about_patch() {
        let current = Version::new(1, 54, 1);
        let max_version = BareVersion::ThreeComponents(1, 54, 0);

        assert!(!include_version(&current, None, Some(max_version).as_ref()));
    }

    #[parameterized(current = {
        50, // -inf <= x <= inf
        50, // 1.50.0 <= x <= inf
        50, // -inf <= x <= 1.50.0
        50, // 1.50.0 <= x <= 1.50.0
        50, // 1.49.0 <= x <= 1.50.0
    }, min = {
        None,
        Some(50),
        None,
        Some(50),
        Some(49),
    }, max = {
        None,
        None,
        Some(50),
        Some(50),
        Some(50),
    })]
    fn test_included_versions(current: u64, min: Option<u64>, max: Option<u64>) {
        let current = Version::new(1, current, 0);
        let min_version = min.map(|m| BareVersion::ThreeComponents(1, m, 0));
        let max_version = max.map(|m| BareVersion::ThreeComponents(1, m, 0));

        assert!(include_version(
            &current,
            min_version.as_ref(),
            max_version.as_ref()
        ));
    }

    #[parameterized(current = {
        50, // -inf <= x <= 1.49.0 : false
        50, // 1.51 <= x <= inf    : false
        50, // 1.51 <= x <= 1.52.0 : false
        50, // 1.48 <= x <= 1.49.0 : false
    }, min = {
        None,
        Some(51),
        Some(51),
        Some(48),
    }, max = {
        Some(49),
        None,
        Some(52),
        Some(49),
    })]
    fn test_excluded_versions(current: u64, min: Option<u64>, max: Option<u64>) {
        let current = Version::new(1, current, 0);
        let min_version = min.map(|m| BareVersion::ThreeComponents(1, m, 0));
        let max_version = max.map(|m| BareVersion::ThreeComponents(1, m, 0));

        assert!(!include_version(
            &current,
            min_version.as_ref(),
            max_version.as_ref()
        ));
    }
}
