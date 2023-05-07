use rust_releases::{Release, ReleaseIndex};

use crate::check::Check;
use crate::config::SearchMethod;
use crate::context::FindContext;
use crate::error::{CargoMSRVError, TResult};
use crate::filter_releases::filter_releases;
use crate::manifest::bare_version::BareVersion;
use crate::msrv::MinimumSupportedRustVersion;
use crate::reporter::event::FindResult;
use crate::reporter::Reporter;
use crate::search_method::{Bisect, FindMinimalSupportedRustVersion, Linear};
use crate::writer::toolchain_file::write_toolchain_file;
use crate::writer::write_msrv::write_msrv;
use crate::{semver, SubCommand};

pub struct Find<'index, C: Check> {
    release_index: &'index ReleaseIndex,
    runner: C,
}

impl<'index, C: Check> Find<'index, C> {
    pub fn new(release_index: &'index ReleaseIndex, runner: C) -> Self {
        Self {
            release_index,
            runner,
        }
    }
}

impl<'index, C: Check> SubCommand for Find<'index, C> {
    type Context = FindContext;
    type Output = semver::Version;

    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        find_msrv(ctx, reporter, self.release_index, &self.runner)
    }
}

fn find_msrv(
    ctx: &FindContext,
    reporter: &impl Reporter,
    release_index: &ReleaseIndex,
    runner: &impl Check,
) -> TResult<semver::Version> {
    let search_result = search(ctx, reporter, release_index, runner)?;

    match &search_result {
        MinimumSupportedRustVersion::NoCompatibleToolchain => {
            info!("no minimal-compatible toolchain found");

            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: ctx.check_command_string(),
            })
        }
        MinimumSupportedRustVersion::Toolchain { toolchain } => {
            info!(
                %toolchain,
                "found minimal-compatible toolchain"
            );

            if ctx.output_toolchain_file() {
                write_toolchain_file(ctx, reporter, toolchain.version())?;
            }

            if ctx.write_msrv() {
                write_msrv(ctx, reporter, toolchain.version())?;
            }

            Ok(toolchain.version().clone())
        }
    }
}

fn search(
    ctx: &FindContext,
    reporter: &impl Reporter,
    index: &ReleaseIndex,
    runner: &impl Check,
) -> TResult<MinimumSupportedRustVersion> {
    let releases = index.releases();
    let included_releases = filter_releases(ctx, releases);

    run_with_search_method(ctx, &included_releases, reporter, runner)
}

fn run_with_search_method(
    ctx: &FindContext,
    included_releases: &[Release],
    reporter: &impl Reporter,
    runner: &impl Check,
) -> TResult<MinimumSupportedRustVersion> {
    let search_method = ctx.search_method;
    info!(?search_method);

    // Run a linear or binary search depending on the configuration
    match search_method {
        SearchMethod::Linear => {
            run_searcher(&Linear::new(runner), included_releases, ctx, reporter)
        }
        SearchMethod::Bisect => {
            run_searcher(&Bisect::new(runner), included_releases, ctx, reporter)
        }
    }
}

fn run_searcher(
    method: &impl FindMinimalSupportedRustVersion,
    releases: &[Release],
    ctx: &FindContext,
    reporter: &impl Reporter,
) -> TResult<MinimumSupportedRustVersion> {
    let minimum_capable = method.find_toolchain(releases, ctx, reporter)?;

    report_outcome(&minimum_capable, releases, ctx, reporter)?;

    Ok(minimum_capable)
}

fn report_outcome(
    minimum_capable: &MinimumSupportedRustVersion,
    releases: &[Release],
    ctx: &FindContext,
    reporter: &impl Reporter,
) -> TResult<()> {
    let (min, max) = min_max_releases(releases)?;

    match minimum_capable {
        MinimumSupportedRustVersion::Toolchain { toolchain } => {
            let version = toolchain.version();

            reporter.report_event(FindResult::new_msrv(version.clone(), ctx, min, max))?;
        }
        MinimumSupportedRustVersion::NoCompatibleToolchain => {
            reporter.report_event(FindResult::none(ctx, min, max))?;
        }
    }

    Ok(())
}

fn min_max_releases(rust_releases: &[Release]) -> TResult<(BareVersion, BareVersion)> {
    let min = rust_releases
        .last()
        .map(|v| v.version())
        .ok_or(CargoMSRVError::RustReleasesEmptyReleaseSet)?;
    let max = rust_releases
        .first()
        .map(|v| v.version())
        .ok_or(CargoMSRVError::RustReleasesEmptyReleaseSet)?;

    Ok((min.into(), max.into()))
}

#[cfg(test)]
mod tests;
