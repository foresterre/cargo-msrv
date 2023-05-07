use crate::check::Check;
use crate::config::SearchMethod;
use crate::msrv::MinimumSupportedRustVersion;
use crate::outcome::Outcome;
use crate::reporter::event::{FindMsrv, Progress};
use crate::reporter::Reporter;
use crate::rust_release::RustRelease;
use crate::search_method::FindMinimalSupportedRustVersion;
use crate::TResult;

pub struct Linear<'runner, R: Check> {
    runner: &'runner R,
}

impl<'runner, R: Check> Linear<'runner, R> {
    pub fn new(runner: &'runner R) -> Self {
        Self { runner }
    }

    fn run_check(runner: &R, release: &RustRelease, _reporter: &impl Reporter) -> TResult<Outcome> {
        let toolchain = release.as_toolchain_spec();
        runner.check(&toolchain)
    }
}

impl<'runner, R: Check> FindMinimalSupportedRustVersion for Linear<'runner, R> {
    fn find_toolchain<'spec>(
        &self,
        search_space: &'spec [RustRelease],
        reporter: &impl Reporter,
    ) -> TResult<MinimumSupportedRustVersion> {
        info!(?search_space);

        reporter.run_scoped_event(FindMsrv::new(SearchMethod::Linear), || {
            let mut last_compatible_index = None;
            let total = search_space.len() as u64;

            for (i, release) in search_space.iter().enumerate() {
                let current = i as u64;
                reporter.report_event(Progress::new(current, total, current + 1))?;

                let outcome = Self::run_check(self.runner, release, reporter)?;

                match outcome {
                    Outcome::Failure(_outcome) => {
                        break;
                    }
                    Outcome::Success(_outcome) => {}
                }

                last_compatible_index = Some(i);
            }

            let msrv = last_compatible_index.map(|i| &search_space[i]);

            Ok(MinimumSupportedRustVersion::from_option(msrv))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::TestRunner;
    use crate::reporter::TestReporterWrapper;
    use crate::{semver, Config, SubcommandId};
    use rust_releases::{Release, ReleaseIndex};
    use std::iter::FromIterator;

    #[test]
    fn none_supported() {
        let config = Config::new(SubcommandId::Find, "my-test-target".to_string());
        let reporter = TestReporterWrapper::default();

        let releases = vec![];

        let runner = TestRunner::with_ok(releases.iter().map(Release::version));
        let index = ReleaseIndex::from_iter(releases);

        let linear_search = Linear::new(&runner);
        let actual = linear_search
            .find_toolchain(index.releases(), &config, reporter.reporter())
            .unwrap();

        let expected = MinimumSupportedRustVersion::NoCompatibleToolchain;

        assert_eq!(actual, expected);
    }

    #[test]
    fn all_supported() {
        let config = Config::new(SubcommandId::Find, "my-test-target".to_string());
        let reporter = TestReporterWrapper::default();

        let releases = vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 54, 0)),
        ];

        let runner = TestRunner::with_ok(releases.iter().map(Release::version));
        let index = ReleaseIndex::from_iter(releases);

        let linear_search = Linear::new(&runner);
        let actual = linear_search
            .find_toolchain(index.releases(), &config, reporter.reporter())
            .unwrap();

        let expected = MinimumSupportedRustVersion::Toolchain {
            toolchain: OwnedToolchainSpec::new(&semver::Version::new(1, 54, 0), "my-test-target"),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn most_recent_only() {
        let config = Config::new(SubcommandId::Find, "my-test-target".to_string());
        let reporter = TestReporterWrapper::default();

        let supported_releases = vec![Release::new_stable(semver::Version::new(1, 56, 0))];

        let index_of_releases = vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 54, 0)),
        ];

        let runner = TestRunner::with_ok(supported_releases.iter().map(Release::version));
        let index = ReleaseIndex::from_iter(index_of_releases);

        let linear_search = Linear::new(&runner);
        let actual = linear_search
            .find_toolchain(index.releases(), &config, reporter.reporter())
            .unwrap();

        let expected = MinimumSupportedRustVersion::Toolchain {
            toolchain: OwnedToolchainSpec::new(&semver::Version::new(1, 56, 0), "my-test-target"),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn least_recent_only_expects_rust_backwards_compat() {
        let config = Config::new(SubcommandId::Find, "my-test-target".to_string());
        let reporter = TestReporterWrapper::default();

        let supported_releases = vec![Release::new_stable(semver::Version::new(1, 54, 0))];

        let index_of_releases = vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 54, 0)),
        ];

        let runner = TestRunner::with_ok(supported_releases.iter().map(Release::version));
        let index = ReleaseIndex::from_iter(index_of_releases);

        let linear_search = Linear::new(&runner);
        let actual = linear_search
            .find_toolchain(index.releases(), &config, reporter.reporter())
            .unwrap();

        // Not 1.54, since we expect that the Rust 1.56 must be able to compile everything that 1.54
        // can.
        let expected = MinimumSupportedRustVersion::NoCompatibleToolchain;

        assert_eq!(actual, expected);
    }

    #[test]
    fn middle_one_only_expects_rust_backwards_compat() {
        let config = Config::new(SubcommandId::Find, "my-test-target".to_string());
        let reporter = TestReporterWrapper::default();

        let supported_releases = vec![Release::new_stable(semver::Version::new(1, 55, 0))];

        let index_of_releases = vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 54, 0)),
        ];

        let runner = TestRunner::with_ok(supported_releases.iter().map(Release::version));
        let index = ReleaseIndex::from_iter(index_of_releases);

        let linear_search = Linear::new(&runner);
        let actual = linear_search
            .find_toolchain(index.releases(), &config, reporter.reporter())
            .unwrap();

        // Not 1.55, since we expect that the Rust 1.56 must be able to compile everything that 1.54
        // can.
        let expected = MinimumSupportedRustVersion::NoCompatibleToolchain;
        assert_eq!(actual, expected);
    }
}
