use bisector::{Bisector, ConvergeTo, Indices, Step};

use crate::compatibility::IsCompatible;
use crate::context::SearchMethod;
use crate::error::NoToolchainsToTryError;
use crate::msrv::MinimumSupportedRustVersion;
use crate::outcome::{Compatibility, Compatible, Incompatible};
use crate::reporter::event::{FindMsrv, Progress};
use crate::reporter::Reporter;
use crate::rust::RustRelease;
use crate::search_method::FindMinimalSupportedRustVersion;
use crate::TResult;

pub struct Bisect<'runner, R: IsCompatible> {
    runner: &'runner R,
}

impl<'runner, R: IsCompatible> Bisect<'runner, R> {
    pub fn new(runner: &'runner R) -> Self {
        Self { runner }
    }

    fn run_check(
        runner: &R,
        release: &RustRelease,
        _reporter: &impl Reporter,
    ) -> TResult<ConvergeTo<Incompatible, Compatible>> {
        let toolchain = release.to_toolchain_spec();

        match runner.is_compatible(&toolchain) {
            Ok(outcome) => match outcome {
                Compatibility::Compatible(outcome) => Ok(ConvergeTo::Right(outcome)),
                Compatibility::Incompatible(outcome) => Ok(ConvergeTo::Left(outcome)),
            },
            Err(err) => Err(err),
        }
    }

    fn show_progress(
        iteration: u64,
        total: u64,
        indices: Indices,
        reporter: &impl Reporter,
    ) -> TResult<()> {
        let current = indices.middle() as u64;

        reporter.report_event(Progress::new(current, total, iteration))?;

        Ok(())
    }
}

impl<'runner, R: IsCompatible> FindMinimalSupportedRustVersion for Bisect<'runner, R> {
    fn find_toolchain(
        &self,
        search_space: &[RustRelease],
        reporter: &impl Reporter,
    ) -> TResult<MinimumSupportedRustVersion> {
        info!(?search_space);

        reporter.run_scoped_event(FindMsrv::new(SearchMethod::Bisect), || {
            let searcher = Bisector::new(search_space);

            let total = search_space.len() as u64;
            let mut iteration = 0_u64;
            let mut indices = Indices::try_from_bisector(&searcher)
                .map_err(|_| NoToolchainsToTryError::new_empty())?;

            let mut last_compatible_index = None;

            while let Step {
                indices: next_indices,
                result: Some(step),
            } = searcher.try_bisect(
                |release| Self::run_check(self.runner, release, reporter),
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

                match Self::run_check(self.runner, converged_to_release, reporter)? {
                    ConvergeTo::Left(_outcome) => {
                        last_compatible_index.map(|i| &search_space[i.middle()])
                    }
                    ConvergeTo::Right(_outcome) => Some(converged_to_release),
                }
            } else {
                last_compatible_index.map(|i| &search_space[i.middle()])
            };

            Ok(MinimumSupportedRustVersion::from_option(msrv))
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_releases::Release;

    use crate::compatibility::TestRunner;
    use crate::reporter::TestReporterWrapper;
    use crate::rust::RustRelease;
    use crate::search_method::FindMinimalSupportedRustVersion;
    use crate::semver;

    use super::Bisect;

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
            semver::Version::new(1, 56, 1)
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
            semver::Version::new(1, 57, 0)
        },
        search_space_of_2_most_recent_one_succeeds = {
            &[
                Release::new_stable(semver::Version::new(1, 58, 0)),
                Release::new_stable(semver::Version::new(1, 57, 0)),
            ], &[
                semver::Version::new(1, 58, 0),
            ],
            semver::Version::new(1, 58, 0)
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
            semver::Version::new(1, 55, 0)
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
            semver::Version::new(1, 56, 0)
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
            semver::Version::new(1, 57, 0)
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
            semver::Version::new(1, 58, 0)
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
            semver::Version::new(1, 54, 0)
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
            semver::Version::new(1, 55, 0)
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
            semver::Version::new(1, 56, 0)
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
            semver::Version::new(1, 57, 0)
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
            semver::Version::new(1, 58, 0)
        },
    )]
    fn find_toolchain_with_bisect(
        search_space: &[Release],
        accept: &[semver::Version],
        expected_msrv: semver::Version,
    ) {
        let runner = TestRunner::with_ok("x", accept);
        let bisect = Bisect::new(&runner);

        let reporter = TestReporterWrapper::default();

        let search_space = search_space
            .iter()
            .map(|r| RustRelease::new(r.clone(), "x", &[]))
            .collect::<Vec<_>>();

        let result = bisect
            .find_toolchain(&search_space, reporter.get())
            .unwrap();

        assert_eq!(result.unwrap_version(), expected_msrv);
    }
}
