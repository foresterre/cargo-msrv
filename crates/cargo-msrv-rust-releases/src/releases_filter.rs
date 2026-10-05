use rust_releases::core::{RustRelease, Stable};
use version_number::{FullVersion, Version};

/// Filter releases based on the given configuration.
pub struct ReleasesFilter<'ctx> {
    include_all_patch_releases: bool,
    minimum_version: Option<&'ctx Version>,
    maximum_version: Option<&'ctx Version>,
}

impl<'ctx> ReleasesFilter<'ctx> {
    /// Initiate a new filter
    pub fn new(
        include_all_patch_releases: bool,
        minimum_version: Option<&'ctx Version>,
        maximum_version: Option<&'ctx Version>,
    ) -> Self {
        Self {
            include_all_patch_releases,
            minimum_version,
            maximum_version,
        }
    }

    /// Filter the given slice of releases, based on the options set for the filter.
    pub fn filter(&self, releases: &[RustRelease<Stable>]) -> Vec<RustRelease<Stable>> {
        let releases = if self.include_all_patch_releases {
            releases.to_vec()
        } else {
            latest_patch_releases(releases)
        };

        // Pre-filter the [min-version:max-version] range
        releases
            .into_iter()
            .filter(|release| {
                let version = release.version().version;
                let current = FullVersion::new(version.major(), version.minor(), version.patch());

                include_version(&current, self.minimum_version, self.maximum_version)
            })
            .collect::<Vec<_>>()
    }
}

fn latest_patch_releases(releases: &[RustRelease<Stable>]) -> Vec<RustRelease<Stable>> {
    let mut latest: Vec<RustRelease<Stable>> = Vec::new();

    for release in releases {
        let version = release.version().version;

        let is_same_minor = latest.last().is_some_and(|previous| {
            let previous = previous.version().version;
            previous.major() == version.major() && previous.minor() == version.minor()
        });

        if !is_same_minor {
            latest.push(release.clone());
        }
    }

    latest
}

fn include_version(
    current: &FullVersion,
    min_version: Option<&Version>,
    max_version: Option<&Version>,
) -> bool {
    match (min_version, &max_version) {
        (Some(min), Some(max)) => {
            min.is_compatible_with(current).is_le() && max.is_compatible_with(current).is_ge()
        }
        (Some(min), None) => min.is_compatible_with(current).is_le(),
        (None, Some(max)) => max.is_compatible_with(current).is_ge(),
        (None, None) => true,
    }
}

#[cfg(test)]
mod tests {
    use parameterized::{ide, parameterized};

    use super::*;

    ide!();

    fn release(major: u64, minor: u64, patch: u64) -> RustRelease<Stable> {
        RustRelease::new(Stable::new(major, minor, patch), None, [])
    }

    #[test]
    fn latest_patch_release_per_minor_version() {
        let releases = [
            release(1, 40, 2),
            release(1, 40, 1),
            release(1, 40, 0),
            release(1, 39, 0),
            release(1, 38, 1),
            release(1, 38, 0),
        ];

        let latest = latest_patch_releases(&releases);

        assert_eq!(
            latest,
            vec![release(1, 40, 2), release(1, 39, 0), release(1, 38, 1)]
        );
    }

    #[test]
    fn max_should_ignore_patch() {
        let current = FullVersion::new(1, 54, 1);
        let max_version = Version::new_base_version(1, 54);

        assert!(include_version(&current, None, Some(max_version).as_ref()));
    }

    #[test]
    fn max_should_be_strict_about_patch() {
        let current = FullVersion::new(1, 54, 1);
        let max_version = Version::new_full_version(1, 54, 0);

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
        let current = FullVersion::new(1, current, 0);
        let min_version = min.map(|m| Version::new_full_version(1, m, 0));
        let max_version = max.map(|m| Version::new_full_version(1, m, 0));

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
        let current = FullVersion::new(1, current, 0);
        let min_version = min.map(|m| Version::new_full_version(1, m, 0));
        let max_version = max.map(|m| Version::new_full_version(1, m, 0));

        assert!(!include_version(
            &current,
            min_version.as_ref(),
            max_version.as_ref()
        ));
    }
}
