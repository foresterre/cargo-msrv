use crate::SubCommand;
use crate::compatibility::{CheckTarget, IsCompatible};
use crate::context::{FindContext, SearchMethod};
use crate::error::{CargoMSRVError, NoToolchainsToTryError, TResult};
use crate::msrv::MinimumSupportedRustVersion;
use crate::reporter::Reporter;
use crate::reporter::event::{
    AuxiliaryOutput, AuxiliaryOutputItem, CheckPackage, Destination, FindResult, ToolchainFileKind,
};
use crate::search_method::{Bisect, FindMinimalSupportedRustVersion, Linear};
use cargo_msrv_context::context::error::Error as ContextError;
use cargo_msrv_context::{Package, SelectedPackage};
use cargo_msrv_rust_releases::releases_filter::ReleasesFilter;
use cargo_msrv_rust_releases::{
    AvailabilityFilter, ExcludedRelease, ReleaseIndex, RustRelease, Stable, to_semver,
};
use cargo_msrv_rust_tools::write_toolchain_file;
use cargo_msrv_search::Error as SearchError;
use version_number::Version;
use write_msrv::write_msrv;

mod write_msrv;

pub struct Find<'index, F> {
    release_index: &'index ReleaseIndex,
    check_for: F,
}

impl<'index, F> Find<'index, F> {
    pub fn new(release_index: &'index ReleaseIndex, check_for: F) -> Self {
        Self {
            release_index,
            check_for,
        }
    }
}

impl<F, C> SubCommand for Find<'_, F>
where
    F: Fn(CheckTarget) -> C,
    C: IsCompatible,
{
    type Context = FindContext;
    type Output = semver::Version;

    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        find_msrv(ctx, reporter, self.release_index, &self.check_for)
    }
}

fn find_msrv<C: IsCompatible>(
    ctx: &FindContext,
    reporter: &impl Reporter,
    release_index: &ReleaseIndex,
    check_for: impl Fn(CheckTarget) -> C,
) -> TResult<semver::Version> {
    let mut found = Vec::new();
    let mut failed_commands = Vec::new();

    for package in ctx.environment.packages_to_check(&ctx.check_cmd) {
        let target = CheckTarget::new(&ctx.check_cmd, &ctx.toolchain, &ctx.environment, package);
        let command = target.run_command.components().join(" ");
        let runner = check_for(target);

        let search_result = match package {
            Some(package) => reporter
                .run_scoped_event(CheckPackage::new(SelectedPackage::from(package)), || {
                    find_package_msrv(ctx, reporter, release_index, Some(package), &runner)
                })?,
            None => find_package_msrv(ctx, reporter, release_index, None, &runner)?,
        };

        match search_result {
            Some(version) => found.push(version),
            None => failed_commands.push(command),
        }
    }

    if !failed_commands.is_empty() {
        return Err(CargoMSRVError::UnableToFindAnyGoodVersion {
            commands: failed_commands,
        });
    }

    let msrv = found
        .into_iter()
        .max()
        .ok_or(CargoMSRVError::Context(ContextError::EmptySelection))?;

    if ctx.write_toolchain_file {
        let path = write_toolchain_file(&msrv, ctx.environment.workspace_root())?;

        reporter.report_event(AuxiliaryOutput::new(
            Destination::file(path),
            AuxiliaryOutputItem::toolchain_file(ToolchainFileKind::Toml),
        ))?;
    }

    Ok(msrv)
}

fn find_package_msrv(
    ctx: &FindContext,
    reporter: &impl Reporter,
    release_index: &ReleaseIndex,
    package: Option<&Package>,
    runner: &impl IsCompatible,
) -> TResult<Option<semver::Version>> {
    let search_result = search(ctx, reporter, release_index, package, runner)?;

    match &search_result {
        MinimumSupportedRustVersion::NoCompatibleToolchain => {
            info!("no minimal-compatible toolchain found");

            Ok(None)
        }
        MinimumSupportedRustVersion::Toolchain { toolchain } => {
            info!(
                %toolchain,
                "found minimal-compatible toolchain"
            );

            if ctx.write_msrv {
                let cargo_toml =
                    package.map_or_else(|| ctx.environment.manifest(), |p| p.manifest_path.clone());

                write_msrv(
                    reporter,
                    Version::new_base_version(toolchain.version().major, toolchain.version().minor),
                    Some(release_index), // Reuse the already obtained index
                    &cargo_toml,
                    package.map(SelectedPackage::from),
                )?;
            }

            Ok(Some(toolchain.version().clone()))
        }
    }
}

fn search(
    ctx: &FindContext,
    reporter: &impl Reporter,
    index: &ReleaseIndex,
    package: Option<&Package>,
    runner: &impl IsCompatible,
) -> TResult<MinimumSupportedRustVersion> {
    let releases = index.releases();

    let min = ctx
        .rust_releases
        .resolve_minimum_version(package.or_else(|| ctx.environment.oldest_edition_package()));

    let releases_filter = ReleasesFilter::new(
        ctx.rust_releases.consider_patch_releases,
        min.as_ref(),
        ctx.rust_releases.maximum_rust_version.as_ref(),
    );

    let included_releases = releases_filter.filter(&releases);

    let availability_filter = AvailabilityFilter::new(
        ctx.toolchain.host,
        ctx.toolchain.target,
        ctx.toolchain.components,
    );
    let available_releases = availability_filter.filter(&included_releases);

    run_with_search_method(
        ctx,
        package,
        available_releases.included(),
        available_releases.excluded(),
        reporter,
        runner,
    )
}

fn run_with_search_method(
    ctx: &FindContext,
    package: Option<&Package>,
    included_releases: &[RustRelease<Stable>],
    excluded_releases: &[ExcludedRelease],
    reporter: &impl Reporter,
    runner: &impl IsCompatible,
) -> TResult<MinimumSupportedRustVersion> {
    let search_method = ctx.search_method;
    info!(?search_method);

    // Run a linear or binary search depending on the configuration
    match search_method {
        SearchMethod::Linear => run_searcher(
            &Linear::new(runner, &ctx.toolchain),
            included_releases,
            excluded_releases,
            ctx,
            package,
            reporter,
        ),
        SearchMethod::Bisect => run_searcher(
            &Bisect::new(runner, &ctx.toolchain),
            included_releases,
            excluded_releases,
            ctx,
            package,
            reporter,
        ),
    }
}

fn run_searcher(
    method: &impl FindMinimalSupportedRustVersion,
    releases: &[RustRelease<Stable>],
    excluded_releases: &[ExcludedRelease],
    ctx: &FindContext,
    package: Option<&Package>,
    reporter: &impl Reporter,
) -> TResult<MinimumSupportedRustVersion> {
    let minimum_capable = method
        .find_toolchain(releases, reporter)
        .map_err(|err| match err {
            SearchError::NoToolchainsToTry(inner) if !inner.has_clues() => {
                CargoMSRVError::NoToolchainsToTry(no_toolchains_to_try(ctx, excluded_releases))
            }
            err => err.into(),
        })?;

    report_outcome(&minimum_capable, releases, ctx, package, reporter)?;

    Ok(minimum_capable)
}

fn no_toolchains_to_try(
    ctx: &FindContext,
    excluded_releases: &[ExcludedRelease],
) -> NoToolchainsToTryError {
    let user_min = ctx.rust_releases.minimum_rust_version.clone();
    let user_max = ctx.rust_releases.maximum_rust_version.clone();

    let error = NoToolchainsToTryError::with_details(user_min, user_max);

    if excluded_releases.is_empty() {
        error
    } else {
        error.with_unavailable_toolchains(
            ctx.toolchain.target,
            ctx.toolchain.components,
            excluded_releases,
        )
    }
}

fn report_outcome(
    minimum_capable: &MinimumSupportedRustVersion,
    releases: &[RustRelease<Stable>],
    ctx: &FindContext,
    package: Option<&Package>,
    reporter: &impl Reporter,
) -> TResult<()> {
    let (min, max) = min_max_releases(releases)?;
    let package = package.map(SelectedPackage::from);

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

            reporter.report_event(
                FindResult::new_msrv(
                    version.clone(),
                    target,
                    minimum_considered,
                    maximum_considered,
                    search_method,
                )
                .with_package(package),
            )?;
        }
        MinimumSupportedRustVersion::NoCompatibleToolchain => {
            reporter.report_event(
                FindResult::none(
                    target,
                    minimum_considered,
                    maximum_considered,
                    search_method,
                )
                .with_package(package),
            )?;
        }
    }

    Ok(())
}

fn min_max_releases(rust_releases: &[RustRelease<Stable>]) -> TResult<(Version, Version)> {
    let min = rust_releases
        .last()
        .map(|v| to_semver(v.version()))
        .ok_or(CargoMSRVError::RustReleasesEmptyReleaseSet)?;
    let max = rust_releases
        .first()
        .map(|v| to_semver(v.version()))
        .ok_or(CargoMSRVError::RustReleasesEmptyReleaseSet)?;

    Ok((
        Version::new_full_version(min.major, min.minor, min.patch),
        Version::new_full_version(max.major, max.minor, max.patch),
    ))
}

#[cfg(test)]
mod tests;
