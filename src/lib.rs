use crate::check::check_with_rust_version;
use crate::cli::cmd_matches;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::{latest_stable_version, RustStableVersion};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

pub mod check;
pub mod cli;
pub mod command;
pub mod config;
pub mod errors;
pub mod fetch;

pub fn run_cargo_msrv() -> TResult<()> {
    let matches = cli::cli().get_matches();
    let config = cmd_matches(&matches)?;

    let latest = latest_stable_version()?;

    let m = Arc::new(MultiProgress::new());
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(200);
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("/|\\- ")
            .template("{spinner:.dim.bold} cargo-msrv: {wide_msg}"),
    );

    let decision = msrv(&config, latest, &pb);

    m.join()?;

    match decision {
        Some(good) => {
            pb.finish_with_message(&format!(
                "MSRV version determined to be: {}",
                good.as_string()
            ));

            Ok(())
        }
        None => {
            pb.finish_at_current_pos();

            Err(CargoMSRVError::UnableToFindAnyGoodVersion)
        }
    }
}

pub fn msrv(
    config: &CmdMatches,
    latest: RustStableVersion,
    pb: &ProgressBar,
) -> Option<RustStableVersion> {
    let mut acceptable: Option<RustStableVersion> = None;

    for minor in (0..=latest.minor()).rev() {
        let current = RustStableVersion::new(latest.major(), minor, 0);

        pb.tick();
        pb.set_message(&format!(
            "checking target: {} on version: {}",
            config.target(),
            current.as_string()
        ));

        if check_with_rust_version(&current, &config).is_err() {
            break;
        } else {
            acceptable = Some(current);
        }
    }

    acceptable
}
