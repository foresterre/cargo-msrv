use crate::check::{check_toolchain, CheckStatus};
use crate::cli::cmd_matches;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::ui::Printer;
use rust_releases::source::{FetchResources, RustChangelog, Source};
use rust_releases::{semver, Channel};

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

    let latest_supported = determine_msrv(&config, &index)?;

    if let MinimalCompatibility::NoCompatibleToolchains = latest_supported {
        Err(CargoMSRVError::UnableToFindAnyGoodVersion {
            command: config.check_command().join(" "),
        })
    } else {
        Ok(())
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
    versions: &rust_releases::index::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let mut compatibility = MinimalCompatibility::NoCompatibleToolchains;
    let releases = versions.releases();
    let cmd = config.check_command().join(" ");

    let ui = Printer::new(releases.len() as u64);
    ui.welcome(config.target(), &cmd);

    for release in releases {
        ui.show_progress("Checking", release.version());
        let status = check_toolchain(release.version(), config, &ui)?;

        if let CheckStatus::Failure { .. } = status {
            break;
        }

        compatibility = status.into();
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
