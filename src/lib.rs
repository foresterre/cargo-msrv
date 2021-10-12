#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_wraps)]

use crate::check::{as_toolchain_specifier, check_toolchain, Outcome};
use crate::config::{Config, ModeIntent, ReleaseSource};
use crate::errors::{CargoMSRVError, TResult};
use crate::reporter::{json, ui};
use crate::reporter::{Output, ProgressAction, __private::NoOutput};
use rust_releases::linear::LatestStableReleases;
use rust_releases::{
    semver, Channel, FetchResources, Release, ReleaseIndex, RustChangelog, RustDist, Source,
};
use std::path::PathBuf;

pub mod check;
pub mod cli;
pub mod command;
pub mod config;
pub mod errors;
pub mod fetch;
pub mod lockfile;
pub mod reporter;

pub fn run_app(config: &Config) -> TResult<()> {
    let index = match config.release_source() {
        ReleaseSource::RustChangelog => {
            RustChangelog::fetch_channel(Channel::Stable)?.build_index()?
        }
        ReleaseSource::RustDist => RustDist::fetch_channel(Channel::Stable)?.build_index()?,
    };

    match config.action_intent() {
        ModeIntent::DetermineMSRV => run_determine_msrv_action(config, &index),
        ModeIntent::VerifyMSRV => run_verify_msrv_action(config, &index),
    }
}

fn run_determine_msrv_action(config: &Config, release_index: &ReleaseIndex) -> TResult<()> {
    match determine_msrv(config, release_index)? {
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

fn run_verify_msrv_action(config: &Config, _release_index: &ReleaseIndex) -> TResult<()> {
    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(CargoMSRVError::Io)?;
    let document =
        decent_toml_rs_alternative::parse_toml(&contents).map_err(CargoMSRVError::ParseToml)?;

    let msrv = document
        .get("package")
        .and_then(|field| field.get("metadata"))
        .and_then(|field| field.get("msrv"))
        .and_then(|value| value.as_string())
        .ok_or(CargoMSRVError::NoMSRVKeyInCargoToml(cargo_toml))?;

    let version = semver::Version::parse(&msrv).map_err(CargoMSRVError::SemverError)?;

    let cmd = config.check_command().join(" ");

    match config.output_format() {
        config::OutputFormat::Human => {
            let reporter = ui::HumanPrinter::new(1, config.target(), &cmd);
            reporter.mode(ModeIntent::VerifyMSRV);
            let status = check_toolchain(&version, config, &reporter)?;
            report_verify_completion(&reporter, status, &cmd);
        }
        config::OutputFormat::Json => {
            let reporter = json::JsonPrinter::new(1, config.target(), &cmd);
            reporter.mode(ModeIntent::VerifyMSRV);
            let status = check_toolchain(&version, config, &reporter)?;
            report_verify_completion(&reporter, status, &cmd);
        }
        config::OutputFormat::None => {
            let reporter = NoOutput;
            reporter.mode(ModeIntent::VerifyMSRV);
            let status = check_toolchain(&version, config, &reporter)?;
            report_verify_completion(&reporter, status, &cmd);
        }
    };

    Ok(())
}

fn report_verify_completion(output: &impl Output, status: Outcome, cmd: &str) {
    if status.is_success() {
        output.finish_success(ModeIntent::VerifyMSRV, status.version());
    } else {
        output.finish_failure(ModeIntent::VerifyMSRV, cmd);
    }
}

/// An enum to represent the minimal compatibility
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl From<Outcome> for MinimalCompatibility {
    fn from(outcome: Outcome) -> Self {
        let version = outcome.version().to_owned();
        let toolchain = outcome.toolchain().to_string();

        if outcome.is_success() {
            MinimalCompatibility::CapableToolchain { version, toolchain }
        } else {
            MinimalCompatibility::NoCompatibleToolchains
        }
    }
}

pub fn determine_msrv(
    config: &Config,
    index: &rust_releases::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let cmd = config.check_command().join(" ");

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

    match config.output_format() {
        config::OutputFormat::Human => {
            let ui = ui::HumanPrinter::new(included_releases.len() as u64, config.target(), &cmd);
            ui.mode(ModeIntent::DetermineMSRV);
            determine_msrv_impl(config, &included_releases, &cmd, &ui)
        }
        config::OutputFormat::Json => {
            let output =
                json::JsonPrinter::new(included_releases.len() as u64, config.target(), &cmd);
            output.mode(ModeIntent::DetermineMSRV);
            determine_msrv_impl(config, &included_releases, &cmd, &output)
        }
        config::OutputFormat::None => {
            let output = NoOutput;
            output.mode(ModeIntent::DetermineMSRV);
            determine_msrv_impl(config, &included_releases, &cmd, &output)
        }
    }
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
            output.finish_success(ModeIntent::DetermineMSRV, version);
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            output.finish_failure(ModeIntent::DetermineMSRV, cmd)
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
        output.progress(ProgressAction::Checking, release.version());
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
        output.progress(ProgressAction::Checking, release.version());

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

pub fn crate_root_folder(config: &Config) -> TResult<PathBuf> {
    if let Some(path) = config.crate_path() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir().map_err(CargoMSRVError::Io)
    }
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
