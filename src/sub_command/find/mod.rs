use rust_releases::{Release, ReleaseIndex};

use crate::compatibility::IsCompatible;
use crate::context::{FindContext, SearchMethod};
use crate::error::{CargoMSRVError, NoToolchainsToTryError, TResult};
use crate::manifest::bare_version::BareVersion;
use crate::msrv::MinimumSupportedRustVersion;
use crate::reporter::event::FindResult;
use crate::reporter::Reporter;
use crate::rust::releases_filter::ReleasesFilter;
use crate::rust::RustRelease;
use crate::search_method::{Bisect, FindMinimalSupportedRustVersion, Linear};
use crate::writer::toolchain_file::write_toolchain_file;
use crate::writer::write_msrv::write_msrv;
use crate::{semver, SubCommand};

pub struct Find<'index, C: IsCompatible> {
    release_index: &'index ReleaseIndex,
    runner: C,
}

impl<'index, C: IsCompatible> Find<'index, C> {
    pub fn new(release_index: &'index ReleaseIndex, runner: C) -> Self {
        Self {
            release_index,
            runner,
        }
    }
}

impl<'index, C: IsCompatible> SubCommand for Find<'index, C> {
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
    runner: &impl IsCompatible,
) -> TResult<semver::Version> {
    let search_result = search(ctx, reporter, release_index, runner)?;

    match &search_result {
        MinimumSupportedRustVersion::NoCompatibleToolchain => {
            info!("no minimal-compatible toolchain found");

            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: ctx.run_command().components().join(" "),
            })
        }
        MinimumSupportedRustVersion::Toolchain { toolchain } => {
            info!(
                %toolchain,
                "found minimal-compatible toolchain"
            );

            let version = toolchain.version();

            if ctx.write_toolchain_file {
                let crate_root = ctx.environment.root();

                write_toolchain_file(reporter, version, crate_root)?;
            }

            if ctx.write_msrv {
                let environment_ctx = ctx.environment.clone();
                let rust_releases_ctx = ctx.rust_releases.clone();

                write_msrv(
                    reporter,
                    BareVersion::two_component_from_semver(toolchain.version()),
                    Some(release_index), // Re-use the already obtained index
                    environment_ctx,
                    rust_releases_ctx,
                )?;
            }

            Ok(version.clone())
        }
    }
}

fn search(
    ctx: &FindContext,
    reporter: &impl Reporter,
    index: &ReleaseIndex,
    runner: &impl IsCompatible,
) -> TResult<MinimumSupportedRustVersion> {
    let releases = index.releases();

    let min = ctx
        .rust_releases
        .resolve_minimum_version(&ctx.environment)?;

    let releases_filter = ReleasesFilter::new(
        ctx.rust_releases.consider_patch_releases,
        min.as_ref(),
        ctx.rust_releases.maximum_rust_version.as_ref(),
    );

    let included_releases = releases_filter.filter(releases);
    run_with_search_method(ctx, &included_releases, reporter, runner)
}

fn run_with_search_method(
    ctx: &FindContext,
    included_releases: &[Release],
    reporter: &impl Reporter,
    runner: &impl IsCompatible,
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
    let searchable_releases = releases
        .iter()
        .map(|r| RustRelease::new(r.clone(), ctx.toolchain.target, ctx.toolchain.components))
        .collect::<Vec<_>>();
    let minimum_capable = method
        .find_toolchain(&searchable_releases, reporter)
        .map_err(|err| match err {
            CargoMSRVError::NoToolchainsToTry(inner) if !inner.has_clues() => {
                let user_min = ctx.rust_releases.minimum_rust_version.clone();
                let user_max = ctx.rust_releases.maximum_rust_version.clone();

                CargoMSRVError::NoToolchainsToTry(NoToolchainsToTryError::with_clues(
                    user_min, user_max,
                ))
            }
            _ => err,
        })?;

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

    let minimum_considered = ctx
        .rust_releases
        .minimum_rust_version
        .clone()
        .unwrap_or(min);

    let maximum_considered = ctx
        .rust_releases
        .maximum_rust_version
        .clone()
        .unwrap_or(max);

    let target = ctx.toolchain.target;
    let search_method = ctx.search_method;

    match minimum_capable {
        MinimumSupportedRustVersion::Toolchain { toolchain } => {
            let version = toolchain.version();

            reporter.report_event(FindResult::new_msrv(
                version.clone(),
                target,
                minimum_considered,
                maximum_considered,
                search_method,
            ))?;
        }
        MinimumSupportedRustVersion::NoCompatibleToolchain => {
            reporter.report_event(FindResult::none(
                target,
                minimum_considered,
                maximum_considered,
                search_method,
            ))?;
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
