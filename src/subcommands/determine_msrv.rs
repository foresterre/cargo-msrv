use rust_releases::linear::LatestStableReleases;
use rust_releases::{semver, Release, ReleaseIndex};

use crate::check::{as_toolchain_specifier, check_toolchain};
use crate::config::{Config, ModeIntent};
use crate::errors::{CargoMSRVError, TResult};
use crate::paths::crate_root_folder;
use crate::reporter::{Output, ProgressAction};
use crate::result::MinimalCompatibility;

pub fn run_determine_msrv_action<R: Output>(
    config: &Config,
    reporter: &R,
    release_index: &ReleaseIndex,
) -> TResult<()> {
    match determine_msrv(config, reporter, release_index)? {
        MinimalCompatibility::NoCompatibleToolchains => {
            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: config.check_command().join(" "),
            })
        }
        MinimalCompatibility::CapableToolchain { ref version, .. }
            if config.output_toolchain_file() =>
        {
            output_toolchain_file(config, version)
        }
        MinimalCompatibility::CapableToolchain { .. } => Ok(()),
    }
}

pub fn determine_msrv<R: Output>(
    config: &Config,
    reporter: &R,
    index: &rust_releases::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let cmd = config.check_command_string();

    let releases = index.releases();

    let releases = if config.include_all_patch_releases() {
        releases.to_vec()
    } else {
        releases
            .to_vec()
            .into_iter()
            .latest_stable_releases()
            .collect()
    };

    // Pre-filter the [min-version:max-version] range
    let included_releases = releases
        .into_iter()
        .filter(|release| {
            include_version(
                release.version(),
                config.minimum_version(),
                config.maximum_version(),
            )
        })
        .collect::<Vec<_>>();

    reporter.mode(ModeIntent::DetermineMSRV);
    reporter.set_steps(included_releases.len() as u64);
    determine_msrv_impl(config, &included_releases, &cmd, reporter)
}

fn determine_msrv_impl(
    config: &Config,
    included_releases: &[Release],
    cmd: &str,
    output: &impl Output,
) -> TResult<MinimalCompatibility> {
    let mut compatibility = MinimalCompatibility::NoCompatibleToolchains;

    output.set_steps(included_releases.len() as u64);

    // Whether to perform a linear (most recent to least recent), or binary search
    if config.bisect() {
        test_against_releases_bisect(included_releases, &mut compatibility, config, output)?;
    } else {
        test_against_releases_linearly(included_releases, &mut compatibility, config, output)?;
    }

    match &compatibility {
        MinimalCompatibility::CapableToolchain {
            toolchain: _,
            version,
        } => {
            output.finish_success(ModeIntent::DetermineMSRV, Some(version));
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            output.finish_failure(ModeIntent::DetermineMSRV, Some(cmd));
        }
    }

    Ok(compatibility)
}

fn test_against_releases_linearly(
    releases: &[Release],
    compatibility: &mut MinimalCompatibility,
    config: &Config,
    output: &impl Output,
) -> TResult<()> {
    for release in releases {
        output.progress(ProgressAction::Checking(release.version()));
        let outcome = check_toolchain(release.version(), config, output)?;

        if !outcome.is_success() {
            break;
        }

        *compatibility = outcome.into();
    }

    Ok(())
}

// Use a binary search to find the MSRV
fn test_against_releases_bisect(
    releases: &[Release],
    compatibility: &mut MinimalCompatibility,
    config: &Config,
    output: &impl Output,
) -> TResult<()> {
    use rust_releases::bisect::{Bisect, Narrow};

    // track progressed items
    let progressed = std::cell::Cell::new(0u64);
    let mut binary_search = Bisect::from_slice(releases);
    let outcome = binary_search.search_with_result_and_remainder(|release, remainder| {
        output.progress(ProgressAction::Checking(release.version()));

        // increment progressed items
        let steps = progressed.replace(progressed.get().saturating_add(1));
        output.set_steps(steps + (remainder as u64));

        let outcome = check_toolchain(release.version(), config, output)?;

        if outcome.is_success() {
            // Expand search space
            TResult::Ok(Narrow::ToRight)
        } else {
            // Shrink search space
            TResult::Ok(Narrow::ToLeft)
        }
    });

    // update compatibility
    *compatibility = outcome?
        .map_or(MinimalCompatibility::NoCompatibleToolchains, |i| {
            let version = releases[i].version();

            MinimalCompatibility::CapableToolchain {
                toolchain: as_toolchain_specifier(version, config.target()),
                version: version.clone(),
            }
        });

    Ok(())
}

fn include_version(
    current: &semver::Version,
    min_version: Option<&semver::Version>,
    max_version: Option<&semver::Version>,
) -> bool {
    match (min_version, max_version) {
        (Some(min), Some(max)) => current >= min && current <= max,
        (Some(min), None) => current >= min,
        (None, Some(max)) => current <= max,
        (None, None) => true,
    }
}

const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

fn output_toolchain_file(config: &Config, stable_version: &semver::Version) -> TResult<()> {
    let path_prefix = crate_root_folder(config)?;

    // check if the rust-toolchain(.toml) file already exists
    if path_prefix.join(TOOLCHAIN_FILE).exists() {
        eprintln!(
            "Not writing toolchain file, '{}' already exists",
            TOOLCHAIN_FILE
        );
        return Ok(());
    } else if path_prefix.join(TOOLCHAIN_FILE_TOML).exists() {
        eprintln!(
            "Not writing toolchain file, '{}' already exists",
            TOOLCHAIN_FILE_TOML
        );
        return Ok(());
    }

    let path = path_prefix.join(TOOLCHAIN_FILE);
    let content = format!(
        r#"[toolchain]
channel = "{}"
"#,
        stable_version
    );

    std::fs::write(&path, content)?;
    eprintln!("Written toolchain file to '{}'", &path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use parameterized::{ide, parameterized};
    use rust_releases::semver::Version;

    use super::*;

    ide!();

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
