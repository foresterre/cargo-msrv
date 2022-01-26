use crate::check::Check;
use crate::outcome::{Outcome, Status};
use crate::search_methods::FindMinimalCapableToolchain;
use crate::toolchain::ToolchainSpec;
use crate::{Config, MinimalCompatibility, Output, ProgressAction, TResult};
use rust_releases::Release;

pub struct Linear<R: Check> {
    runner: R,
}

impl<R: Check> Linear<R> {
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    fn run_check(
        runner: &R,
        release: &Release,
        config: &Config,
        output: &impl Output,
    ) -> TResult<Outcome> {
        output.progress(ProgressAction::Checking(release.version()));

        let toolchain = ToolchainSpec::new(config.target(), release.version());
        runner.check(config, &toolchain)
    }

    fn minimum_capable(
        releases: &[Release],
        index_of_msrv: Option<usize>,
        last_error: Option<String>,
        config: &Config,
    ) -> MinimalCompatibility {
        index_of_msrv.map_or(MinimalCompatibility::NoCompatibleToolchains, |i| {
            let version = releases[i].version();

            MinimalCompatibility::CapableToolchain {
                toolchain: ToolchainSpec::new(config.target(), version)
                    .spec()
                    .to_string(),
                version: version.clone(),
                last_error,
            }
        })
    }
}

impl<R: Check> FindMinimalCapableToolchain for Linear<R> {
    fn find_toolchain(
        &self,
        search_space: &[Release],
        config: &Config,
        output: &impl Output,
    ) -> TResult<MinimalCompatibility> {
        let mut last_compatible_index = None;
        let mut last_failure_report = None;

        for (i, release) in search_space.iter().enumerate() {
            let outcome = Self::run_check(&self.runner, release, config, output)?;

            if let Status::Failure(msg) = outcome.status() {
                last_failure_report = Some(msg);
                break;
            }

            last_compatible_index = Some(i);
        }

        Ok(Self::minimum_capable(
            search_space,
            last_compatible_index,
            last_failure_report,
            config,
        ))
    }
}
