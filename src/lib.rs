extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::check::{check_toolchain, CheckStatus};
use crate::cli::cmd_matches;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::{latest_stable_version, RustStableVersion};

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

    let latest = latest_stable_version()?;

    let latest_supported = determine_msrv(&config, latest)?;

    match latest_supported {
        MinimalCompatibility::CapableToolchain { version, .. } => {
            info!(
                "Minimum Supported Rust Version (MSRV) determined to be: {}",
                version.as_string()
            );

            Ok(())
        }
        MinimalCompatibility::None { latest_toolchain } => {
            Err(CargoMSRVError::UnableToFindAnyGoodVersion { latest_toolchain })
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
        version: RustStableVersion,
    },
    /// Compatibility is none, if the check on the last available toolchain fails
    None {
        // last known toolchain specifier
        latest_toolchain: String,
    },
}

impl MinimalCompatibility {
    pub fn unwrap_version(&self) -> RustStableVersion {
        if let Self::CapableToolchain { version, .. } = self {
            return *version;
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
            CheckStatus::Failure { toolchain, .. } => MinimalCompatibility::None {
                latest_toolchain: toolchain,
            },
        }
    }
}

pub fn determine_msrv(
    config: &CmdMatches,
    latest: RustStableVersion,
) -> TResult<MinimalCompatibility> {
    let minors = (0..=latest.minor()).rev();
    let mut compatibility = MinimalCompatibility::None {
        latest_toolchain: latest.as_toolchain_string(config.target()),
    };

    for minor in minors {
        let current = RustStableVersion::new(latest.major(), minor, 0);
        let status = run_check_for_version(config, current)?;

        if let CheckStatus::Failure { .. } = status {
            break;
        }

        compatibility = status.into();
    }

    Ok(compatibility)
}

fn run_check_for_version(
    config: &CmdMatches,
    current_version: RustStableVersion,
) -> TResult<CheckStatus> {
    info!(
        "checking target '{}' using Rust version '{}'",
        config.target(),
        current_version.as_string()
    );

    check_toolchain(current_version, config)
}
