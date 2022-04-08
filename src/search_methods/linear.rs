use crate::check::Check;
use crate::outcome::Outcome;
use crate::reporter::{write_failed_check, write_succeeded_check};
use crate::result::MinimalCompatibility;
use crate::search_methods::FindMinimalCapableToolchain;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, Output, ProgressAction, TResult};
use rust_releases::Release;

pub struct Linear<'runner, R: Check> {
    runner: &'runner R,
}

impl<'runner, R: Check> Linear<'runner, R> {
    pub fn new(runner: &'runner R) -> Self {
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
        config: &Config,
    ) -> MinimalCompatibility {
        index_of_msrv.map_or(MinimalCompatibility::NoCompatibleToolchains, |i| {
            let version = releases[i].version();

            MinimalCompatibility::CapableToolchain {
                toolchain: OwnedToolchainSpec::new(version, config.target()),
            }
        })
    }
}

impl<'runner, R: Check> FindMinimalCapableToolchain for Linear<'runner, R> {
    fn find_toolchain<'spec>(
        &self,
        search_space: &'spec [Release],
        config: &'spec Config,
        output: &impl Output,
    ) -> TResult<MinimalCompatibility> {
        let mut last_compatible_index = None;

        for (i, release) in search_space.iter().enumerate() {
            let outcome = Self::run_check(self.runner, release, config, output)?;

            match outcome {
                Outcome::Failure(outcome) => {
                    write_failed_check(&outcome, config, output);
                    break;
                }
                Outcome::Success(outcome) => {
                    write_succeeded_check(&outcome, config, output);
                }
            }

            last_compatible_index = Some(i);
        }

        Ok(Self::minimum_capable(
            search_space,
            last_compatible_index,
            config,
        ))
    }
}
