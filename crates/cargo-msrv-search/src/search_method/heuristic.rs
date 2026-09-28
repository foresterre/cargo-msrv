use crate::TResult;
use crate::error::NoToolchainsToTryError;
use crate::msrv::MinimumSupportedRustVersion;
use crate::search_method::FindMinimalSupportedRustVersion;
use cargo_msrv_context::{SearchMethod, ToolchainContext};
use cargo_msrv_reporter::Reporter;
use cargo_msrv_reporter::event::FindMsrv;
use cargo_msrv_rust_releases::{RustRelease, Stable, to_semver};
use cargo_msrv_types::Toolchain;

/// Takes the estimated lower bound of the MSRV as the MSRV, without checking any toolchain.
///
/// The MSRV is the least recent release which satisfies the lower bound. Since the lower
/// bound is only an estimate, the MSRV may be wrong.
pub struct Heuristic<'ctx> {
    toolchain_ctx: &'ctx ToolchainContext,
    lower_bound: Option<semver::Version>,
}

impl<'ctx> Heuristic<'ctx> {
    /// Without a `lower_bound`, the MSRV is the least recent release.
    pub fn new(
        toolchain_ctx: &'ctx ToolchainContext,
        lower_bound: Option<semver::Version>,
    ) -> Self {
        Self {
            toolchain_ctx,
            lower_bound,
        }
    }

    fn toolchain_for(&self, release: &RustRelease<Stable>) -> Toolchain {
        Toolchain::new(
            to_semver(release.version()),
            self.toolchain_ctx.target,
            self.toolchain_ctx.components,
        )
    }

    /// The least recent release which satisfies the lower bound, if any.
    ///
    /// The search space is ordered from most to least recent.
    fn least_recent_satisfying<'r>(
        &self,
        search_space: &'r [RustRelease<Stable>],
    ) -> Option<&'r RustRelease<Stable>> {
        match &self.lower_bound {
            None => search_space.last(),
            Some(lower_bound) => search_space
                .iter()
                .rev()
                .find(|release| to_semver(release.version()) >= *lower_bound),
        }
    }
}

impl FindMinimalSupportedRustVersion for Heuristic<'_> {
    fn find_toolchain(
        &self,
        search_space: &[RustRelease<Stable>],
        reporter: &impl Reporter,
    ) -> TResult<MinimumSupportedRustVersion> {
        info!(?search_space, lower_bound = ?self.lower_bound);

        if search_space.is_empty() {
            return Err(NoToolchainsToTryError::new_empty().into());
        }

        reporter.run_scoped_event(FindMsrv::new(SearchMethod::Heuristic), || {
            let msrv = self
                .least_recent_satisfying(search_space)
                .map(|release| self.toolchain_for(release));

            Ok(MinimumSupportedRustVersion::from_option(msrv))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_msrv_reporter::TestReporterWrapper;

    const TOOLCHAIN_CTX: ToolchainContext = ToolchainContext {
        host: "x",
        target: "x",
        components: &[],
    };

    fn releases() -> Vec<RustRelease<Stable>> {
        (60..=70)
            .rev()
            .map(|minor| RustRelease::new(Stable::new(1, minor, 0), None, []))
            .collect()
    }

    fn search(lower_bound: Option<semver::Version>) -> MinimumSupportedRustVersion {
        let reporter = TestReporterWrapper::default();

        Heuristic::new(&TOOLCHAIN_CTX, lower_bound)
            .find_toolchain(&releases(), reporter.get())
            .unwrap()
    }

    #[yare::parameterized(
        at_release = { Some(semver::Version::new(1, 65, 0)), 65 },
        between_releases = { Some(semver::Version::new(1, 64, 1)), 65 },
        below_all_releases = { Some(semver::Version::new(1, 50, 0)), 60 },
        most_recent_release = { Some(semver::Version::new(1, 70, 0)), 70 },
        no_lower_bound = { None, 60 },
    )]
    fn msrv_is_least_recent_release_satisfying_lower_bound(
        lower_bound: Option<semver::Version>,
        minor: u64,
    ) {
        assert_eq!(
            search(lower_bound).unwrap_version(),
            semver::Version::new(1, minor, 0)
        );
    }

    #[test]
    fn lower_bound_above_all_releases() {
        assert_eq!(
            search(Some(semver::Version::new(1, 90, 0))),
            MinimumSupportedRustVersion::NoCompatibleToolchain
        );
    }

    #[test]
    fn empty_search_space() {
        let reporter = TestReporterWrapper::default();

        let result = Heuristic::new(&TOOLCHAIN_CTX, None).find_toolchain(&[], reporter.get());

        assert!(result.is_err());
    }
}
