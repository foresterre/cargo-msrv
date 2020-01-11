extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::check::check_with_rust_version;
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

    let latest_supported = msrv(&config, latest)?;

    match latest_supported {
        Some(ok) => {
            info!(
                "Minimum Supported Rust Version (MSRV) determined to be: {}",
                ok.as_string()
            );

            Ok(())
        }
        None => Err(CargoMSRVError::UnableToFindAnyGoodVersion),
    }
}

pub fn msrv(config: &CmdMatches, latest: RustStableVersion) -> TResult<Option<RustStableVersion>> {
    let mut acceptable: Option<RustStableVersion> = None;

    for minor in (0..=latest.minor()).rev() {
        let current = RustStableVersion::new(latest.major(), minor, 0);

        info!(
            "checking target '{}' using Rust version '{}'",
            config.target(),
            current.as_string()
        );

        if let Err(err) = check_with_rust_version(&current, &config) {
            match err {
                // This version doesn't work, so we quit the loop.
                // Then 'acceptable' (may) contain the last successfully checked version.
                CargoMSRVError::RustupRunWithCommandFailed => break,
                // In this case an error occurred during the check, so we want to report the error
                // instead of reporting the last ok version.
                _ => return Err(err),
            }
        } else {
            acceptable = Some(current);
        }
    }

    Ok(acceptable)
}
