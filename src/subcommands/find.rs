use rust_releases::{Release, ReleaseIndex};

use crate::check::RunCheck;
use crate::config::{Config, ModeIntent, SearchMethod};
use crate::errors::{CargoMSRVError, TResult};
use crate::releases::filter_releases;
use crate::reporter::Output;
use crate::result::MinimalCompatibility;
use crate::search_methods::{Bisect, FindMinimalCapableToolchain, Linear};
use crate::toolchain_file::write_toolchain_file;

pub fn run_find_msrv_action<R: Output>(
    config: &Config,
    reporter: &R,
    release_index: &ReleaseIndex,
) -> TResult<()> {
    match find_msrv(config, reporter, release_index)? {
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
        } => {
            info!(
                %toolchain,
                %version,
                "found minimal-compatible toolchain"
            );

            if config.output_toolchain_file() {
                write_toolchain_file(config, version)?;
            }

            Ok(())
        }
    }
}

pub fn find_msrv<R: Output>(
    config: &Config,
    reporter: &R,
    index: &rust_releases::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let releases = index.releases();
    let included_releases = filter_releases(config, releases);

    reporter.mode(ModeIntent::Find);
    reporter.set_steps(included_releases.len() as u64);
    run_with_search_method(config, &included_releases, reporter)
}

fn run_with_search_method(
    config: &Config,
    included_releases: &[Release],
    output: &impl Output,
) -> TResult<MinimalCompatibility> {
    output.set_steps(included_releases.len() as u64);

    let search_method = config.search_method();
    info!(?search_method);

    let runner = RunCheck::new(output);

    // Run a linear or binary search depending on the configuration
    match search_method {
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
            output.finish_success(ModeIntent::Find, Some(version));
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            output.finish_failure(ModeIntent::Find, Some(cmd));
        }
    }
}
