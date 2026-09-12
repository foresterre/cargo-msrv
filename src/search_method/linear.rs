use crate::TResult;
use crate::compatibility::IsCompatible;
use crate::context::{SearchMethod, ToolchainContext};
use crate::error::NoToolchainsToTryError;
use crate::msrv::MinimumSupportedRustVersion;
use crate::outcome::Compatibility;
use crate::reporter::Reporter;
use crate::reporter::event::{FindMsrv, Progress};
use crate::rust::{RustRelease, Stable, Toolchain, to_semver};
use crate::search_method::FindMinimalSupportedRustVersion;

pub struct Linear<'runner, 'ctx, R: IsCompatible> {
    runner: &'runner R,
    toolchain_ctx: &'ctx ToolchainContext,
}

impl<'runner, 'ctx, R: IsCompatible> Linear<'runner, 'ctx, R> {
    pub fn new(runner: &'runner R, toolchain_ctx: &'ctx ToolchainContext) -> Self {
        Self {
            runner,
            toolchain_ctx,
        }
    }

    fn toolchain_for(&self, release: &RustRelease<Stable>) -> Toolchain {
        Toolchain::new(
            to_semver(release.version()),
            self.toolchain_ctx.target,
            self.toolchain_ctx.components,
        )
    }

    fn run_check(
        &self,
        release: &RustRelease<Stable>,
        _reporter: &impl Reporter,
    ) -> TResult<Compatibility> {
        self.runner.is_compatible(&self.toolchain_for(release))
    }
}

impl<R: IsCompatible> FindMinimalSupportedRustVersion for Linear<'_, '_, R> {
    fn find_toolchain(
        &self,
        search_space: &[RustRelease<Stable>],
        reporter: &impl Reporter,
    ) -> TResult<MinimumSupportedRustVersion> {
        info!(?search_space);

        if search_space.is_empty() {
            return Err(NoToolchainsToTryError::new_empty().into());
        }

        reporter.run_scoped_event(FindMsrv::new(SearchMethod::Linear), || {
            let mut last_compatible_index = None;
            let total = search_space.len() as u64;

            for (i, release) in search_space.iter().enumerate() {
                let current = i as u64;
                reporter.report_event(Progress::new(current, total, current + 1))?;

                let outcome = self.run_check(release, reporter)?;

                match outcome {
                    Compatibility::Incompatible(_outcome) => {
                        break;
                    }
                    Compatibility::Compatible(_outcome) => {}
                }

                last_compatible_index = Some(i);
            }

            let msrv = last_compatible_index.map(|i| self.toolchain_for(&search_space[i]));

            Ok(MinimumSupportedRustVersion::from_option(msrv))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compatibility::TestRunner;
    use crate::reporter::TestReporterWrapper;
    use crate::rust::ReleaseIndex;
    use std::iter::FromIterator;

    fn toolchain_context() -> ToolchainContext {
        ToolchainContext {
            host: "x",
            target: "x",
            components: &[],
        }
    }

    fn accepted_versions(releases: &[RustRelease<Stable>]) -> Vec<semver::Version> {
        releases.iter().map(|r| to_semver(r.version())).collect()
    }

    #[test]
    fn none_supported() {
        let reporter = TestReporterWrapper::default();

        let runner = TestRunner::with_ok("x", &[]);
        let index = ReleaseIndex::from_iter(vec![
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 54, 0), None, []),
        ]);

        let toolchain = toolchain_context();
        let linear_search = Linear::new(&runner, &toolchain);

        let search_space = index.releases();
        let actual = linear_search
            .find_toolchain(&search_space, reporter.get())
            .unwrap();

        let expected = MinimumSupportedRustVersion::NoCompatibleToolchain;

        assert_eq!(actual, expected);
    }

    #[test]
    fn all_supported() {
        let reporter = TestReporterWrapper::default();

        let releases = vec![
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 54, 0), None, []),
        ];

        let accepted = accepted_versions(&releases);
        let runner = TestRunner::with_ok("x", accepted.iter());
        let index = ReleaseIndex::from_iter(releases);

        let toolchain = toolchain_context();
        let linear_search = Linear::new(&runner, &toolchain);

        let search_space = index.releases();
        let actual = linear_search
            .find_toolchain(&search_space, reporter.get())
            .unwrap();

        let expected = MinimumSupportedRustVersion::Toolchain {
            toolchain: Toolchain::new(semver::Version::new(1, 54, 0), "x", &[]),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn most_recent_only() {
        let reporter = TestReporterWrapper::default();

        let supported_releases = [RustRelease::new(Stable::new(1, 56, 0), None, [])];

        let index_of_releases = vec![
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 54, 0), None, []),
        ];

        let accepted = accepted_versions(&supported_releases);
        let runner = TestRunner::with_ok("x", accepted.iter());
        let index = ReleaseIndex::from_iter(index_of_releases);

        let toolchain = toolchain_context();
        let linear_search = Linear::new(&runner, &toolchain);

        let search_space = index.releases();
        let actual = linear_search
            .find_toolchain(&search_space, reporter.get())
            .unwrap();

        let expected = MinimumSupportedRustVersion::Toolchain {
            toolchain: Toolchain::new(semver::Version::new(1, 56, 0), "x", &[]),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn least_recent_only_expects_rust_backwards_compat() {
        let reporter = TestReporterWrapper::default();

        let supported_releases = [RustRelease::new(Stable::new(1, 54, 0), None, [])];

        let index_of_releases = vec![
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 54, 0), None, []),
        ];

        let accepted = accepted_versions(&supported_releases);
        let runner = TestRunner::with_ok("x", accepted.iter());
        let index = ReleaseIndex::from_iter(index_of_releases);

        let toolchain = toolchain_context();
        let linear_search = Linear::new(&runner, &toolchain);

        let search_space = index.releases();
        let actual = linear_search
            .find_toolchain(&search_space, reporter.get())
            .unwrap();

        // Not 1.54, since we expect that the Rust 1.56 must be able to compile everything that 1.54
        // can.
        let expected = MinimumSupportedRustVersion::NoCompatibleToolchain;

        assert_eq!(actual, expected);
    }

    #[test]
    fn middle_one_only_expects_rust_backwards_compat() {
        let reporter = TestReporterWrapper::default();

        let supported_releases = [RustRelease::new(Stable::new(1, 55, 0), None, [])];

        let index_of_releases = [
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 54, 0), None, []),
        ];

        let accepted = accepted_versions(&supported_releases);
        let runner = TestRunner::with_ok("x", accepted.iter());
        let index = ReleaseIndex::from_iter(index_of_releases);

        let toolchain = toolchain_context();
        let linear_search = Linear::new(&runner, &toolchain);

        let search_space = index.releases();
        let actual = linear_search
            .find_toolchain(&search_space, reporter.get())
            .unwrap();

        // Not 1.55, since we expect that the Rust 1.56 must be able to compile everything that 1.54
        // can.
        let expected = MinimumSupportedRustVersion::NoCompatibleToolchain;
        assert_eq!(actual, expected);
    }
}
