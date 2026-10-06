use crate::compatibility::{CheckTarget, IsCompatible};
use crate::error::{CargoMSRVError, TResult};
use crate::outcome::Compatibility;
use crate::reporter::Reporter;
use crate::reporter::event::{CheckPackage, VerifyResult};
use crate::sub_command::SubCommand;
use cargo_msrv_context::context::verify::{RustVersion, RustVersionSource, VerifyCheck};
use cargo_msrv_context::{SelectedPackage, VerifyContext};
use cargo_msrv_rust_releases::{ReleaseIndex, to_semver};
use cargo_msrv_types::{Toolchain, find_matching_version};
use std::fmt;
use version_number::Version;

/// Verifier which determines whether a given Rust version is deemed compatible or not.
pub struct Verify<'index, F> {
    release_index: &'index ReleaseIndex,
    check_for: F,
}

impl<'index, F> Verify<'index, F> {
    /// Instantiate the verifier using a release index and a way to create a runner for each check.
    ///
    /// The runner is used to determine whether a given Rust version will be deemed compatible or not.
    pub fn new(release_index: &'index ReleaseIndex, check_for: F) -> Self {
        Self {
            release_index,
            check_for,
        }
    }
}

impl<F, C> SubCommand for Verify<'_, F>
where
    F: Fn(CheckTarget) -> C,
    C: IsCompatible,
{
    type Context = VerifyContext;
    type Output = ();

    /// Run the verifier against a Rust version which is obtained from the config.
    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        let mut failures = Vec::new();

        for check in &ctx.checks {
            let package = check.package.as_ref();
            let target =
                CheckTarget::new(&ctx.check_cmd, &ctx.toolchain, &ctx.environment, package);
            let runner = (self.check_for)(target);

            let verified = match package {
                Some(package) => reporter
                    .run_scoped_event(CheckPackage::new(SelectedPackage::from(package)), || {
                        verify_msrv(reporter, ctx, self.release_index, check, &runner)
                    })?,
                None => verify_msrv(reporter, ctx, self.release_index, check, &runner)?,
            };

            if let Err(failed) = verified {
                failures.push(failed);
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(CargoMSRVError::SubCommandVerify(Error::VerifyFailed(
                VerifyFailures(failures),
            )))
        }
    }
}

/// Verify whether a Cargo project is compatible with a `rustup run` command,
/// for the (given or specified) `rust_version`.
fn verify_msrv(
    reporter: &impl Reporter,
    ctx: &VerifyContext,
    release_index: &ReleaseIndex,
    check: &VerifyCheck,
    runner: &impl IsCompatible,
) -> TResult<Result<(), VerifyFailed>> {
    let rust_version = &check.rust_version;
    let available = release_index
        .stable_releases()
        .iter()
        .map(|release| to_semver(release.version()))
        .collect::<Vec<_>>();
    let version = find_matching_version(rust_version.version(), available.iter())?;

    let target = ctx.toolchain.target;
    let components = ctx.toolchain.components;
    let toolchain = Toolchain::new(version.clone(), target, components);
    let package = check.package.as_ref().map(SelectedPackage::from);

    match runner.is_compatible(&toolchain)? {
        Compatibility::Compatible(_) => {
            reporter.report_event(VerifyResult::compatible(toolchain).with_package(package))?;
            Ok(Ok(()))
        }
        Compatibility::Incompatible(f) => {
            reporter.report_event(
                VerifyResult::incompatible(toolchain, Some(f.error_message))
                    .with_package(package.clone()),
            )?;
            Ok(Err(VerifyFailed::new(package, rust_version.clone())))
        }
    }
}

/// Error which can be returned if the verifier deemed the tested Rust version incompatible.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    VerifyFailed(VerifyFailures),
}

#[derive(Debug)]
pub struct VerifyFailures(Vec<VerifyFailed>);

impl VerifyFailures {
    pub fn failures(&self) -> &[VerifyFailed] {
        &self.0
    }
}

impl fmt::Display for VerifyFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self
            .0
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        f.write_str(&lines)
    }
}

/// Data structure which contains information about which version failed to verify, and where
/// we obtained this version from.
///
/// It is combination of the Rust version which was tested for compatibility and the source which was
/// used to find this tested Rust version.
#[derive(Debug)]
pub struct VerifyFailed {
    package: Option<SelectedPackage>,
    rust_version: Version,
    source: RustVersionSource,
}

impl VerifyFailed {
    fn new(package: Option<SelectedPackage>, rust_version: RustVersion) -> Self {
        let (rust_version, source) = rust_version.into_parts();

        Self {
            package,
            rust_version,
            source,
        }
    }

    pub fn package(&self) -> Option<&SelectedPackage> {
        self.package.as_ref()
    }
}

impl fmt::Display for VerifyFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.package {
            Some(package) => write!(
                f,
                "Package '{}' was found to be incompatible with Rust version '{}' specified {}",
                package.name, self.rust_version, self.source
            ),
            None => write!(
                f,
                "Crate source was found to be incompatible with Rust version '{}' specified {}",
                self.rust_version, self.source
            ),
        }
    }
}
