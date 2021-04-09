#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_wraps)]

use crate::check::{as_toolchain_specifier, check_toolchain, CheckStatus};
use crate::cli::cmd_matches;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::ui::Printer;
use rust_releases::source::{FetchResources, RustChangelog, Source};
use rust_releases::{semver, Channel, Release};

pub mod check;
pub mod cli;
pub mod command;
pub mod config;
pub mod errors;
pub mod fetch;
pub mod ui;

pub fn run_cargo_msrv() -> TResult<()> {
    let matches = cli::cli().get_matches();
    let config = cmd_matches(&matches)?;

    let index_strategy = RustChangelog::fetch_channel(Channel::Stable)?;
    let index = index_strategy.build_index()?;

    match determine_msrv(&config, &index)? {
        MinimalCompatibility::NoCompatibleToolchains => {
            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: config.check_command().join(" "),
            })
        }
        MinimalCompatibility::CapableToolchain { ref version, .. }
            if config.output_toolchain_file() =>
        {
            output_toolchain_file(&config, version)
        }
        MinimalCompatibility::CapableToolchain { .. } => Ok(()),
    }
}

/// An enum to represent the minimal compatibility
#[derive(Clone, Debug)]
pub enum MinimalCompatibility {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    CapableToolchain {
        // toolchain specifier
        toolchain: String,
        // checked Rust version
        version: semver::Version,
    },
    /// Compatibility is none, if the check on the last available toolchain fails
    NoCompatibleToolchains,
}

impl MinimalCompatibility {
    pub fn unwrap_version(&self) -> semver::Version {
        if let Self::CapableToolchain { version, .. } = self {
            return version.clone();
        }

        panic!("Unable to unwrap MinimalCompatibility (CapableToolchain::version)")
    }
}

impl From<CheckStatus> for MinimalCompatibility {
    fn from(from: CheckStatus) -> Self {
        match from {
            CheckStatus::Success { version, toolchain } => {
                MinimalCompatibility::CapableToolchain { version, toolchain }
            }
            CheckStatus::Failure { toolchain: _, .. } => {
                MinimalCompatibility::NoCompatibleToolchains
            }
        }
    }
}

pub fn determine_msrv(
    config: &CmdMatches,
    index: &rust_releases::index::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let mut compatibility = MinimalCompatibility::NoCompatibleToolchains;
    let cmd = config.check_command().join(" ");

    let releases = index.releases();
    let ui = Printer::new(releases.len() as u64);
    ui.welcome(config.target(), &cmd);

    // The collecting step is necessary, because Rust can't deal with equal opaque types
    let releases = if config.include_all_patch_releases() {
        index.all_releases_iterator().collect::<Vec<_>>()
    } else {
        index.stable_releases_iterator().collect::<Vec<_>>()
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
        .cloned()
        .collect::<Vec<_>>();

    ui.set_progress_bar_length(included_releases.len() as u64);

    // Whether to perform a linear (most recent to least recent), or binary search
    if config.bisect() {
        test_against_releases_bisect(&included_releases, &mut compatibility, config, &ui)?;
    } else {
        test_against_releases_linearly(&included_releases, &mut compatibility, config, &ui)?;
    }

    match &compatibility {
        MinimalCompatibility::CapableToolchain {
            toolchain: _,
            version,
        } => {
            ui.finish_with_ok(&version);
        }
        MinimalCompatibility::NoCompatibleToolchains => ui.finish_with_err(&cmd),
    }

    Ok(compatibility)
}

fn test_against_releases_linearly(
    releases: &[Release],
    compatibility: &mut MinimalCompatibility,
    config: &CmdMatches,
    ui: &Printer,
) -> TResult<()> {
    for release in releases {
        ui.show_progress("Checking", release.version());
        let status = check_toolchain(release.version(), config, ui)?;

        if let CheckStatus::Failure { .. } = status {
            break;
        }

        *compatibility = status.into();
    }

    Ok(())
}

// Use a binary search to find the MSRV
fn test_against_releases_bisect(
    releases: &[Release],
    compatibility: &mut MinimalCompatibility,
    config: &CmdMatches,
    ui: &Printer,
) -> TResult<()> {
    use rust_releases::index::{Bisect, Narrow};

    // track progressed items
    let progressed = std::cell::Cell::new(0u64);
    let mut binary_search = Bisect::from_slice(&releases);
    let outcome = binary_search.search_with_result_and_remainder(|release, remainder| {
        ui.show_progress("Checking", release.version());

        // increment progressed items
        let steps = progressed.replace(progressed.get().saturating_add(1));
        ui.set_progress_bar_length(steps + (remainder as u64));

        let status = check_toolchain(release.version(), config, ui)?;

        match status {
            CheckStatus::Failure { .. } => TResult::Ok(Narrow::ToLeft),
            CheckStatus::Success { .. } => TResult::Ok(Narrow::ToRight),
        }
    });

    // update compatibility
    *compatibility = outcome?
        .map(|i| {
            let version = releases[i].version();

            MinimalCompatibility::CapableToolchain {
                toolchain: as_toolchain_specifier(version, config.target()),
                version: version.clone(),
            }
        })
        .unwrap_or(MinimalCompatibility::NoCompatibleToolchains);

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

fn output_toolchain_file(config: &CmdMatches, stable_version: &semver::Version) -> TResult<()> {
    let path_prefix = if let Some(path) = config.crate_path() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?
    };

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
    use super::*;
    use parameterized::{ide, parameterized};
    use rust_releases::semver::Version;

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
