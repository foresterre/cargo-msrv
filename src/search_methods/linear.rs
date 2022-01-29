use crate::check::Check;
use crate::outcome::{FailureOutcome, Outcome};
use crate::search_methods::FindMinimalCapableToolchain;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
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

        let toolchain = ToolchainSpec::new(release.version(), config.target());
        runner.check(config, &toolchain)
    }

    fn minimum_capable(
        releases: &[rust_releases::Release],
        index_of_msrv: Option<usize>,
        last_error: Option<FailureOutcome>,
        config: &Config,
    ) -> MinimalCompatibility {
        index_of_msrv.map_or(MinimalCompatibility::NoCompatibleToolchains, |i| {
            let version = releases[i].version();

            MinimalCompatibility::CapableToolchain {
                toolchain: OwnedToolchainSpec::new(version, config.target()),
                last_error,
            }
        })
    }
}

impl<R: Check> FindMinimalCapableToolchain for Linear<R> {
    fn find_toolchain<'spec>(
        &self,
        search_space: &'spec [Release],
        config: &'spec Config,
        output: &impl Output,
    ) -> TResult<MinimalCompatibility> {
        let mut last_compatible_index = None;
        let mut last_failure_report = None;

        for (i, release) in search_space.iter().enumerate() {
            let outcome = Self::run_check(&self.runner, release, config, output)?;

            if let Outcome::Failure(reason) = outcome {
                last_failure_report = Some(reason);
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
