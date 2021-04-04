#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms)]

use crate::check::{check_toolchain, CheckStatus};
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

    if config.include_all_patch_releases() {
        test_against_releases(
            index.all_releases_iterator(),
            &mut compatibility,
            config,
            &ui,
        )?;
    } else {
        test_against_releases(
            index.stable_releases_iterator(),
            &mut compatibility,
            config,
            &ui,
        )?;
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

fn test_against_releases<'release, I>(
    releases: I,
    compatibility: &mut MinimalCompatibility,
    config: &CmdMatches,
    ui: &Printer,
) -> TResult<()>
where
    I: Iterator<Item = &'release Release>,
{
    for release in releases {
        // releases are ordered high to low; if we have reached a version which is below the minimum,
        // we can stop.
        if let Some(min) = config.minimum_version() {
            if release.version() < min {
                break;
            }
        }

        // releases are ordered high to low; if we find a version which is higher than the maximum,
        // we can skip over it.
        if let Some(max) = config.maximum_version() {
            if release.version() > max {
                ui.skip_version(release.version());
                continue;
            }
        }

        ui.show_progress("Checking", release.version());
        let status = check_toolchain(release.version(), config, ui)?;

        if let CheckStatus::Failure { .. } = status {
            break;
        }

        *compatibility = status.into();
    }

    Ok(())
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
