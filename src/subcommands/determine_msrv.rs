use rust_releases::linear::LatestStableReleases;
use rust_releases::{semver, Release, ReleaseIndex};

use crate::check::RunCheck;
use crate::config::{Config, ModeIntent, SearchMethod};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::paths::crate_root_folder;
use crate::reporter::Output;
use crate::result::MinimalCompatibility;
use crate::search_methods::{Bisect, FindMinimalCapableToolchain, Linear};

pub fn run_determine_msrv_action<R: Output>(
    config: &Config,
    reporter: &R,
    release_index: &ReleaseIndex,
) -> TResult<()> {
    match determine_msrv(config, reporter, release_index)? {
        MinimalCompatibility::NoCompatibleToolchains => {
            info!("no minimal-compatible toolchain found");

            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: config.check_command().join(" "),
            })
        }
        MinimalCompatibility::CapableToolchain {
            toolchain,
            ref version,
            last_error: _,
        } if config.output_toolchain_file() => {
            let version_formatted = version.to_string();
            info!(
                toolchain = toolchain.as_str(),
                version = version_formatted.as_str(),
                "found minimal-compatible toolchain"
            );

            output_toolchain_file(config, version)
        }
        MinimalCompatibility::CapableToolchain {
            toolchain,
            version,
            last_error: _,
        } => {
            let version = version.to_string();

            info!(
                toolchain = toolchain.as_str(),
                version = version.as_str(),
                "found minimal-compatible toolchain"
            );

            Ok(())
        }
    }
}

pub fn determine_msrv<R: Output>(
    config: &Config,
    reporter: &R,
    index: &rust_releases::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let releases = index.releases();
    let included_releases = filter_releases(config, releases);

    reporter.mode(ModeIntent::DetermineMSRV);
    reporter.set_steps(included_releases.len() as u64);
    determine_msrv_impl(config, &included_releases, reporter)
}

fn filter_releases(config: &Config, releases: &[Release]) -> Vec<Release> {
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

fn determine_msrv_impl(
    config: &Config,
    included_releases: &[Release],
    output: &impl Output,
) -> TResult<MinimalCompatibility> {
    output.set_steps(included_releases.len() as u64);
    info!(search_method = ?config.search_method());

    // Whether to perform a linear (most recent to least recent), or binary search
    let runner = RunCheck::new(output);

    match config.search_method() {
        SearchMethod::Linear => {
            run_searcher(Linear::new(runner), included_releases, config, output)
        }
        SearchMethod::Bisect => {
            run_searcher(Bisect::new(runner), included_releases, config, output)
        }
    }
}

fn run_searcher(
    method: impl FindMinimalCapableToolchain,
    releases: &[Release],
    config: &Config,
    output: &impl Output,
) -> TResult<MinimalCompatibility> {
    let minimum_capable = method.find_toolchain(releases, config, output)?;

    let cmd = config.check_command_string();
    report_outcome(&minimum_capable, &cmd, output);

    Ok(minimum_capable)
}

fn report_outcome(minimum_capable: &MinimalCompatibility, cmd: &str, output: &impl Output) {
    match minimum_capable {
        MinimalCompatibility::CapableToolchain {
            toolchain: _,
            version,
            last_error: _,
        } => {
            output.finish_success(ModeIntent::DetermineMSRV, Some(version));
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            output.finish_failure(ModeIntent::DetermineMSRV, Some(cmd));
        }
    }
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

    std::fs::write(&path, content).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::WriteFile(path.clone()),
    })?;
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
