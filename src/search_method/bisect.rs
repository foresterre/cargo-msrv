use bisector::{Bisector, ConvergeTo, Indices, Step};
use rust_releases::Release;

use crate::check::Check;
use crate::error::NoToolchainsToTryError;
use crate::msrv::MinimumSupportedRustVersion;
use crate::outcome::{FailureOutcome, Outcome, SuccessOutcome};
use crate::reporter::event::{FindMsrv, Progress};
use crate::reporter::EventReporter;
use crate::search_method::FindMinimalSupportedRustVersion;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, TResult};

pub struct Bisect<'runner, R: Check> {
    runner: &'runner R,
}

impl<'runner, R: Check> Bisect<'runner, R> {
    pub fn new(runner: &'runner R) -> Self {
        Self { runner }
    }

    fn run_check(
        runner: &R,
        release: &Release,
        config: &Config,
        _reporter: &impl EventReporter,
    ) -> TResult<ConvergeTo<FailureOutcome, SuccessOutcome>> {
        let toolchain = ToolchainSpec::new(release.version(), config.target());
        match runner.check(config, &toolchain) {
            Ok(outcome) => match outcome {
                Outcome::Success(outcome) => Ok(ConvergeTo::Right(outcome)),
                Outcome::Failure(outcome) => Ok(ConvergeTo::Left(outcome)),
            },
            Err(err) => Err(err),
        }
    }

    fn show_progress(
        iteration: u64,
        total: u64,
        indices: Indices,
        reporter: &impl EventReporter,
    ) -> TResult<()> {
        let current = indices.middle() as u64;

        reporter.report_event(Progress::new(current, total, iteration))?;

        Ok(())
    }

    fn minimum_capable(msrv: Option<&Release>, config: &Config) -> MinimumSupportedRustVersion {
        msrv.map_or(
            MinimumSupportedRustVersion::NoCompatibleToolchain,
            |release| {
                let version = release.version();

                MinimumSupportedRustVersion::Toolchain {
                    toolchain: OwnedToolchainSpec::new(version, config.target()),
                }
            },
        )
    }
}

impl<'runner, R: Check> FindMinimalSupportedRustVersion for Bisect<'runner, R> {
    fn find_toolchain(
        &self,
        search_space: &[Release],
        config: &Config,
        reporter: &impl EventReporter,
    ) -> TResult<MinimumSupportedRustVersion> {
        reporter.run_scoped_event(FindMsrv::new(config.search_method()), || {
            let searcher = Bisector::new(search_space);

            let total = search_space.len() as u64;
            let mut iteration = 0_u64;
            let mut indices =
                Indices::try_from_bisector(&searcher).map_err(|_| NoToolchainsToTryError {
                    min: config.minimum_version().map(Clone::clone),
                    max: config.maximum_version().map(Clone::clone),
                    search_space: search_space.to_vec(),
                })?;

            let mut last_compatible_index = None;

            info!(?search_space);

            while let Step {
                indices: next_indices,
                result: Some(step),
            } = searcher.try_bisect(
                |release| Self::run_check(self.runner, release, config, reporter),
                indices,
            )? {
                iteration += 1;

                info!(?indices, ?next_indices);

                Self::show_progress(iteration, total, indices, reporter)?;

                match step {
                    ConvergeTo::Left(_outcome) => {}
                    ConvergeTo::Right(_outcome) => {
                        last_compatible_index = Some(indices);
                    }
                }

                indices = next_indices;
            }

            let converged_to_release = &search_space[indices.middle()];

            // Work-around for regression:
            // https://github.com/foresterre/cargo-msrv/issues/288
            let msrv = if indices.middle() == search_space.len() - 1 {
                Self::show_progress(iteration + 1, total, indices, reporter)?;

                match Self::run_check(self.runner, converged_to_release, config, reporter)? {
                    ConvergeTo::Left(_outcome) => {
                        last_compatible_index.map(|i| &search_space[i.middle()])
                    }
                    ConvergeTo::Right(_outcome) => Some(converged_to_release),
                }
            } else {
                last_compatible_index.map(|i| &search_space[i.middle()])
            };

            Ok(Self::minimum_capable(msrv, config))
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_releases::Release;

    use crate::check::TestRunner;
    use crate::reporter::TestReporterWrapper;
    use crate::search_method::FindMinimalSupportedRustVersion;
    use crate::semver::Version;
    use crate::{semver, Config, SubcommandId};

    use super::Bisect;

    fn fake_config() -> Config<'static> {
        Config::new(SubcommandId::Find, "".to_string())
    }

    #[yare::parameterized(
        regression288_search_space_of_3_all_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 1)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 1)),
            ], &[
                semver::Version::new(1, 58, 1),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 56, 1),
            ],
            Version::new(1, 56, 1)
        },
        one_option = {
            &[
                Release::new_stable(semver::Version::new(1, 56, 1)),
            ],
            &[
                semver::Version::new(1, 56, 1)
            ],
            semver::Version::new(1, 56, 1)
        },
        search_space_of_3_most_recent_two_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
            ],
            &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
            ],
            semver::Version::new(1, 57, 0)
        },
        search_space_of_3_most_recent_one_succeeds = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
            ],
            &[
                semver::Version::new(1, 58, 0),
            ],
            semver::Version::new(1, 58, 0)
        },

        search_space_of_2_all_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
            ],
            Version::new(1, 57, 0)
        },
        search_space_of_2_most_recent_one_succeeds = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
            ],
            Version::new(1, 58, 0)
        },
        search_space_of_4_all_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 56, 0),
                semver::Version::new(1, 55, 0),
            ],
            Version::new(1, 55, 0)
        },
        search_space_of_4_most_recent_three_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 56, 0),
            ],
            Version::new(1, 56, 0)
        },
        search_space_of_4_most_recent_two_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
            ],
            Version::new(1, 57, 0)
        },
        search_space_of_4_most_recent_one_succeeds = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
            ],
            Version::new(1, 58, 0)
        },
        search_space_of_5_all_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
                Release::new_stable(semver::Version::new(1, 54, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 56, 0),
                semver::Version::new(1, 55, 0),
                semver::Version::new(1, 54, 0),
            ],
            Version::new(1, 54, 0)
        },
        search_space_of_5_most_recent_four_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
                Release::new_stable(semver::Version::new(1, 54, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 56, 0),
                semver::Version::new(1, 55, 0),
            ],
            Version::new(1, 55, 0)
        },
        search_space_of_5_most_recent_three_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
                Release::new_stable(semver::Version::new(1, 54, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 56, 0),
            ],
            Version::new(1, 56, 0)
        },
        search_space_of_5_most_recent_two_succeed = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
                Release::new_stable(semver::Version::new(1, 54, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
                semver::Version::new(1, 57, 0),
            ],
            Version::new(1, 57, 0)
        },
        search_space_of_5_most_recent_one_succeeds = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
                Release::new_stable(semver::Version::new(1, 56, 0)),
                Release::new_stable(semver::Version::new(1, 55, 0)),
                Release::new_stable(semver::Version::new(1, 54, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
            ],
            Version::new(1, 58, 0)
        },
    )]
    fn find_toolchain_with_bisect(
        search_space: &[Release],
        accept: &[Version],
        expected_msrv: Version,
    ) {
        let runner = TestRunner::with_ok(accept);
        let bisect = Bisect::new(&runner);

        let reporter = TestReporterWrapper::default();

        let result = bisect
            .find_toolchain(search_space, &fake_config(), reporter.reporter())
            .unwrap();

        assert_eq!(result.unwrap_version(), expected_msrv);
    }
}
