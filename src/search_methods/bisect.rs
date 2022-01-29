use crate::check::Check;
use crate::outcome::{FailureOutcome, Outcome};
use crate::search_methods::FindMinimalCapableToolchain;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, MinimalCompatibility, Output, ProgressAction, TResult};
use bisector::{Bisector, ConvergeTo, Indices, Step};
use rust_releases::Release;

pub struct Bisect<R: Check> {
    runner: R,
}

impl<R: Check> Bisect<R> {
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    fn run_check(
        runner: &R,
        release: &Release,
        config: &Config,
        output: &impl Output,
    ) -> TResult<ConvergeTo<FailureOutcome, ()>> {
        output.progress(ProgressAction::Checking(release.version()));

        let toolchain = ToolchainSpec::new(release.version(), config.target());
        match runner.check(config, &toolchain) {
            Ok(outcome) => match outcome {
                Outcome::Success(_) => Ok(ConvergeTo::Right(())),
                Outcome::Failure(outcome) => Ok(ConvergeTo::Left(outcome)),
            },
            Err(err) => Err(err),
        }
    }

    fn update_progress_bar(iteration: u64, indices: Indices, output: &impl Output) {
        let remainder = (indices.right - indices.left) as u64;
        output.set_steps(remainder + iteration);
    }

    fn minimum_capable(
        releases: &[Release],
        index_of_msrv: Option<Indices>,
        last_error: Option<FailureOutcome>,
        config: &Config,
    ) -> MinimalCompatibility {
        index_of_msrv.map_or(MinimalCompatibility::NoCompatibleToolchains, |i| {
            let version = releases[i.middle()].version();

            MinimalCompatibility::CapableToolchain {
                toolchain: OwnedToolchainSpec::new(version, config.target()),
                last_error,
            }
        })
    }
}

impl<R: Check> FindMinimalCapableToolchain for Bisect<R> {
    fn find_toolchain(
        &self,
        search_space: &[Release],
        config: &Config,
        output: &impl Output,
    ) -> TResult<MinimalCompatibility> {
        let searcher = Bisector::new(search_space);

        let mut iteration = 0_u64;
        let mut indices = Indices::from_bisector(&searcher);

        let mut last_compatible_index = None;
        let mut last_failure_report = None;

        while let Step {
            indices: next_indices,
            result: Some(step),
        } = searcher.try_bisect(
            |release| Self::run_check(&self.runner, release, config, output),
            indices,
        )? {
            iteration += 1;

            Self::update_progress_bar(iteration, next_indices, output);

            match step {
                ConvergeTo::Left(outcome) => {
                    last_failure_report = Some(outcome);
                }
                ConvergeTo::Right(_) => {
                    last_compatible_index = Some(indices);
                }
            }

            indices = next_indices;
        }

        Ok(Self::minimum_capable(
            search_space,
            last_compatible_index,
            last_failure_report,
            config,
        ))
    }
}
