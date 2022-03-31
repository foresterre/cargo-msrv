use crate::{semver, Config};
use rust_releases::linear::LatestStableReleases;
use rust_releases::Release;

pub fn filter_releases(config: &Config, releases: &[Release]) -> Vec<Release> {
    let releases = if config.include_all_patch_releases() {
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
                config.minimum_version(),
                config.maximum_version(),
            )
        })
        .collect::<Vec<_>>()
}

fn include_version(
    current: &semver::Version,
    min_version: Option<&semver::Version>,
    max_version: Option<&semver::Version>,
) -> bool {
    // Workaround for #278:
    //
    // What we consider here is that a maximum version of x.y should match `x.y.*`.
    //
    // Possibly this function should take BareVersion's as arguments; currently we however don't
    // and take full semver compatible versions.
    //
    // What we need here is a kind of reverse semver Tilde comparator.
    // Where Tilde allows patch updates, with greater than comparisons, we need 'allow patch updates'
    // but with less than comparisons.
    //
    // For this work around, we let the comparison ignore patch versions for max,
    // by setting the patch version equal to the current version, ensuring for every current version,
    // the given patch version will be acceptable.
    //
    // Inadvertently however, we may also allow larger patch versions which are precisely specified
    // as a full semver version, e.g. x.y.z now also matches x.y.f where f > z, since we'll
    // set f = z in this workaround.
    let max_version = max_version.map(|v| {
        if current.patch > v.patch {
            semver::Version::new(v.major, v.minor, current.patch)
        } else {
            v.clone()
        }
    });

    match (min_version, &max_version) {
        (Some(min), Some(max)) => current >= min && current <= max,
        (Some(min), None) => current >= min,
        (None, Some(max)) => current <= max, // todo,
        (None, None) => true,
    }
}

#[cfg(test)]
mod tests {
    use parameterized::{ide, parameterized};
    use rust_releases::semver::Version;

    use super::*;

    ide!();

    #[test]
    fn max_should_ignore_patch() {
        let current = Version::new(1, 54, 1);
        let max_version = Version::new(1, 54, 0);

        assert!(include_version(&current, None, Some(max_version).as_ref()));
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
        let min_version = min.map(|m| Version::new(1, m, 0));
        let max_version = max.map(|m| Version::new(1, m, 0));

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
        let min_version = min.map(|m| Version::new(1, m, 0));
        let max_version = max.map(|m| Version::new(1, m, 0));

        assert!(!include_version(
            &current,
            min_version.as_ref(),
            max_version.as_ref()
        ));
    }
}
