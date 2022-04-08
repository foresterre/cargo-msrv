use bisector::{Bisector, ConvergeTo, Indices, Step};
use rust_releases::Release;

use crate::check::Check;
use crate::outcome::{FailureOutcome, Outcome, SuccessOutcome};
use crate::reporter::{write_failed_check, write_succeeded_check};
use crate::result::MinimalCompatibility;
use crate::search_methods::FindMinimalCapableToolchain;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, Output, ProgressAction, TResult};

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
        output: &impl Output,
    ) -> TResult<ConvergeTo<FailureOutcome, SuccessOutcome>> {
        output.progress(ProgressAction::Checking(release.version()));

        let toolchain = ToolchainSpec::new(release.version(), config.target());
        match runner.check(config, &toolchain) {
            Ok(outcome) => match outcome {
                Outcome::Success(outcome) => Ok(ConvergeTo::Right(outcome)),
                Outcome::Failure(outcome) => Ok(ConvergeTo::Left(outcome)),
            },
            Err(err) => Err(err),
        }
    }

    fn update_progress_bar(iteration: u64, indices: Indices, output: &impl Output) {
        let remainder = (indices.right - indices.left) as u64;
        output.set_steps(remainder + iteration);
    }

    fn minimum_capable(msrv: Option<&Release>, config: &Config) -> MinimalCompatibility {
        msrv.map_or(MinimalCompatibility::NoCompatibleToolchains, |release| {
            let version = release.version();

            MinimalCompatibility::CapableToolchain {
                toolchain: OwnedToolchainSpec::new(version, config.target()),
            }
        })
    }
}

impl<'runner, R: Check> FindMinimalCapableToolchain for Bisect<'runner, R> {
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

        info!(?search_space);

        while let Step {
            indices: next_indices,
            result: Some(step),
        } = searcher.try_bisect(
            |release| Self::run_check(self.runner, release, config, output),
            indices,
        )? {
            iteration += 1;

            info!(?indices, ?next_indices);

            Self::update_progress_bar(iteration, next_indices, output);

            match step {
                ConvergeTo::Left(outcome) => {
                    write_failed_check(&outcome, config, output);
                }
                ConvergeTo::Right(outcome) => {
                    last_compatible_index = Some(indices);
                    write_succeeded_check(&outcome, config, output);
                }
            }

            indices = next_indices;
        }

        let converged_to_release = &search_space[indices.middle()];

        // Work-around for regression:
        // https://github.com/foresterre/cargo-msrv/issues/288
        let msrv = if indices.middle() == search_space.len() - 1 {
            match Self::run_check(self.runner, converged_to_release, config, output)? {
                ConvergeTo::Left(outcome) => {
                    write_failed_check(&outcome, config, output);
                    last_compatible_index.map(|i| &search_space[i.middle()])
                }
                ConvergeTo::Right(outcome) => {
                    write_succeeded_check(&outcome, config, output);
                    Some(converged_to_release)
                }
            }
        } else {
            last_compatible_index.map(|i| &search_space[i.middle()])
        };

        Ok(Self::minimum_capable(msrv, config))
    }
}

#[cfg(test)]
mod tests {
    use rust_releases::Release;

    use crate::reporter::no_output::NoOutput;
    use crate::search_methods::FindMinimalCapableToolchain;
    use crate::semver::Version;
    use crate::testing::TestRunner;
    use crate::{semver, Config, ModeIntent};

    use super::Bisect;

    fn fake_config() -> Config<'static> {
        Config::new(ModeIntent::Find, "".to_string())
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
        accept: &[semver::Version],
        expected_msrv: semver::Version,
    ) {
        let runner = TestRunner::with_ok(accept);
        let bisect = Bisect::new(&runner);

        let result = bisect
            .find_toolchain(search_space, &fake_config(), &NoOutput {})
            .unwrap();

        assert_eq!(result.unwrap_version(), expected_msrv);
    }
}
