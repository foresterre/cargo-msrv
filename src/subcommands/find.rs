use rust_releases::{Release, ReleaseIndex};
use storyteller::Reporter;

use crate::check::Check;
use crate::config::{Config, ModeIntent, SearchMethod};
use crate::errors::{CargoMSRVError, TResult};
use crate::releases::filter_releases;
use crate::result::MinimalCompatibility;
use crate::search_methods::{Bisect, FindMinimalCapableToolchain, Linear};
use crate::writers::toolchain_file::write_toolchain_file;
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
    fn run(&self, config: &Config, reporter: &impl Reporter) -> TResult<()> {
        find_msrv(config, reporter, self.release_index, &self.runner)
    }
}

fn find_msrv(
    config: &Config,
    reporter: &impl Reporter,
    release_index: &ReleaseIndex,
    runner: &impl Check,
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

fn search(
    config: &Config,
    reporter: &impl Reporter,
    index: &ReleaseIndex,
    runner: &impl Check,
) -> TResult<MinimalCompatibility> {
    let releases = index.releases();
    let included_releases = filter_releases(config, releases);

    // todo!
    // reporter.mode(ModeIntent::Find);
    // reporter.set_steps(included_releases.len() as u64);

    run_with_search_method(config, &included_releases, reporter, runner)
}

fn run_with_search_method(
    config: &Config,
    included_releases: &[Release],
    reporter: &impl Reporter,
    runner: &impl Check,
) -> TResult<MinimalCompatibility> {
    // todo!
    // reporter.set_steps(included_releases.len() as u64);

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

fn run_searcher(
    method: &impl FindMinimalCapableToolchain,
    releases: &[Release],
    config: &Config,
    reporter: &impl Reporter,
) -> TResult<MinimalCompatibility> {
    let minimum_capable = method.find_toolchain(releases, config, reporter)?;

    report_outcome(&minimum_capable, config, reporter);

    Ok(minimum_capable)
}

fn report_outcome(
    minimum_capable: &MinimalCompatibility,
    config: &Config,
    reporter: &impl Reporter,
) {
    match minimum_capable {
        MinimalCompatibility::CapableToolchain { toolchain } => {
            // todo!
            // output.finish_success(ModeIntent::Find, Some(toolchain.version()));
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            // todo!
            // output.finish_failure(ModeIntent::Find, Some(&config.check_command_string()));
        }
    }
}

#[cfg(test)]
mod tests;
