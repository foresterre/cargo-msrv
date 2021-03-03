extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::check::{check_toolchain, CheckStatus};
use crate::cli::cmd_matches;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use rust_releases::source::{FetchResources, RustChangelog, Source};
use rust_releases::{semver, Channel};

pub mod check;
pub mod cli;
pub mod command;
pub mod config;
pub mod errors;
pub mod fetch;

pub fn init_logger() {
    pretty_env_logger::init()
}

pub fn run_cargo_msrv() -> TResult<()> {
    let matches = cli::cli().get_matches();
    let config = cmd_matches(&matches)?;

    let index_strategy = RustChangelog::fetch_channel(Channel::Stable)?;
    let index = index_strategy.build_index()?;

    let latest_supported = determine_msrv(&config, &index)?;

    match latest_supported {
        MinimalCompatibility::CapableToolchain { version, .. } => {
            info!(
                "Minimum Supported Rust Version (MSRV) determined to be: {}",
                version
            );

            Ok(())
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                latest_toolchain: "TODO latest toolchain".to_string(),
                command: config.check_command().join(" "),
            })
        }
    }
}

/// An enum to represent the minimal compatibility
#[derive(Debug)]
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
            CheckStatus::Success { version, toolchain } => MinimalCompatibility::CapableToolchain {
                version: version.clone(),
                toolchain,
            },
            CheckStatus::Failure { toolchain: _, .. } => {
                MinimalCompatibility::NoCompatibleToolchains
            }
        }
    }
}

pub fn determine_msrv(
    config: &CmdMatches,
    versions: &rust_releases::index::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let mut compatibility = MinimalCompatibility::NoCompatibleToolchains;

    for release in versions.releases() {
        let status = run_check_for_version(config, release.version())?;

        if let CheckStatus::Failure { .. } = status {
            break;
        }

        compatibility = status.into();
    }

    Ok(compatibility)
}

fn run_check_for_version(
    config: &CmdMatches,
    current_version: &semver::Version,
) -> TResult<CheckStatus> {
    info!(
        "checking target '{}' using Rust version '{}'",
        config.target(),
        current_version
    );

    check_toolchain(current_version, config)
}
