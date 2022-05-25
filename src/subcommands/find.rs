use rust_releases::{Release, ReleaseIndex};

use crate::check::Check;
use crate::config::{Config, ModeIntent, SearchMethod};
use crate::errors::{CargoMSRVError, TResult};
use crate::releases::filter_releases;
use crate::reporter::Output;
use crate::result::MinimalCompatibility;
use crate::search_methods::{Bisect, FindMinimalCapableToolchain, Linear};
use crate::toolchain_file::write_toolchain_file;
use crate::writers::write_msrv::write_msrv;
use crate::SubCommand;

pub struct Find<'index, C: Check> {
    release_index: &'index ReleaseIndex,
    runner: C,
}

impl<'index, C: Check> Find<'index, C> {
    pub fn new(release_index: &'index ReleaseIndex, runner: C) -> Self {
        Self {
            release_index,
            runner,
        }
    }
}

impl<'index, C: Check> SubCommand for Find<'index, C> {
    fn run<R: Output>(&self, config: &Config, reporter: &R) -> TResult<()> {
        find_msrv(config, reporter, self.release_index, &self.runner)
    }
}

fn find_msrv<R: Output, C: Check>(
    config: &Config,
    reporter: &R,
    release_index: &ReleaseIndex,
    runner: &C,
) -> TResult<()> {
    match search(config, reporter, release_index, runner)? {
        MinimalCompatibility::NoCompatibleToolchains => {
            info!("no minimal-compatible toolchain found");

            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: config.check_command().join(" "),
            })
        }
        MinimalCompatibility::CapableToolchain { toolchain } => {
            info!(
                %toolchain,
                "found minimal-compatible toolchain"
            );

            if config.output_toolchain_file() {
                write_toolchain_file(config, toolchain.version())?;
            }

            if config.write_msrv() {
                write_msrv(config, reporter, toolchain.version())?;
            }

            Ok(())
        }
    }
}

fn search<R: Output, C: Check>(
    config: &Config,
    reporter: &R,
    index: &rust_releases::ReleaseIndex,
    runner: &C,
) -> TResult<MinimalCompatibility> {
    let releases = index.releases();
    let included_releases = filter_releases(config, releases);

    reporter.mode(ModeIntent::Find);
    reporter.set_steps(included_releases.len() as u64);

    run_with_search_method(config, &included_releases, reporter, runner)
}

fn run_with_search_method<R: Output, C: Check>(
    config: &Config,
    included_releases: &[Release],
    reporter: &R,
    runner: &C,
) -> TResult<MinimalCompatibility> {
    reporter.set_steps(included_releases.len() as u64);

    let search_method = config.search_method();
    info!(?search_method);

    // Run a linear or binary search depending on the configuration
    match search_method {
        SearchMethod::Linear => {
            run_searcher(&Linear::new(runner), included_releases, config, reporter)
        }
        SearchMethod::Bisect => {
            run_searcher(&Bisect::new(runner), included_releases, config, reporter)
        }
    }
}

fn run_searcher<R: Output>(
    method: &impl FindMinimalCapableToolchain,
    releases: &[Release],
    config: &Config,
    reporter: &R,
) -> TResult<MinimalCompatibility> {
    let minimum_capable = method.find_toolchain(releases, config, reporter)?;

    report_outcome(&minimum_capable, config, reporter);

    Ok(minimum_capable)
}

fn report_outcome(minimum_capable: &MinimalCompatibility, config: &Config, output: &impl Output) {
    match minimum_capable {
        MinimalCompatibility::CapableToolchain { toolchain } => {
            output.finish_success(ModeIntent::Find, Some(toolchain.version()));
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            output.finish_failure(ModeIntent::Find, Some(&config.check_command_string()));
        }
    }
}

#[cfg(test)]
mod tests;
