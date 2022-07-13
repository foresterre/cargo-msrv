use rust_releases::Release;

use crate::check::Check;
use crate::msrv::MinimumSupportedRustVersion;
use crate::outcome::Outcome;
use crate::reporter::event::FindMsrv;
use crate::reporter::Reporter;
use crate::search_method::FindMinimalSupportedRustVersion;
use crate::toolchain::{OwnedToolchainSpec, ToolchainSpec};
use crate::{Config, TResult};

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
        _reporter: &impl Reporter,
    ) -> TResult<Outcome> {
        // todo!
        // output.progress(ProgressAction::Checking(release.version()));

        let toolchain = ToolchainSpec::new(release.version(), config.target());
        runner.check(config, &toolchain)
    }

    fn minimum_capable(
        releases: &[Release],
        index_of_msrv: Option<usize>,
        config: &Config,
    ) -> MinimumSupportedRustVersion {
        index_of_msrv.map_or(MinimumSupportedRustVersion::NoCompatibleToolchain, |i| {
            let version = releases[i].version();

            MinimumSupportedRustVersion::Toolchain {
                toolchain: OwnedToolchainSpec::new(version, config.target()),
            }
        })
    }
}

impl<'runner, R: Check> FindMinimalSupportedRustVersion for Linear<'runner, R> {
    fn find_toolchain<'spec>(
        &self,
        search_space: &'spec [Release],
        config: &'spec Config,
        reporter: &impl Reporter,
    ) -> TResult<MinimumSupportedRustVersion> {
        reporter.run_scoped_event(FindMsrv::new(config.search_method()), || {
            let mut last_compatible_index = None;

            for (i, release) in search_space.iter().enumerate() {
                let outcome = Self::run_check(self.runner, release, config, reporter)?;

                match outcome {
                    Outcome::Failure(_outcome) => {
                        break;
                    }
                    Outcome::Success(_outcome) => {}
                }

                last_compatible_index = Some(i);
            }

            Ok(Self::minimum_capable(
                search_space,
                last_compatible_index,
                config,
            ))
        })
    }
}
