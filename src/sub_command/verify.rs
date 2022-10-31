use std::convert::TryFrom;
use std::path::PathBuf;

use rust_releases::{Release, ReleaseIndex};

use crate::check::Check;
use crate::config::Config;
use crate::error::{CargoMSRVError, TResult};
use crate::manifest::bare_version::BareVersion;
use crate::manifest::reader::{DocumentReader, TomlDocumentReader};
use crate::manifest::CargoManifest;
use crate::outcome::Outcome;
use crate::reporter::event::VerifyResult;
use crate::reporter::Reporter;
use crate::sub_command::SubCommand;
use crate::toolchain::ToolchainSpec;

/// Verifier which determines whether a given Rust version is deemed compatible or not.
pub struct Verify<'index, C: Check> {
    release_index: &'index ReleaseIndex,
    runner: C,
}

impl<'index, C: Check> Verify<'index, C> {
    /// Instantiate the verifier using a release index and a runner.
    ///
    /// The runner is used to determine whether a given Rust version will be deemed compatible or not.
    pub fn new(release_index: &'index ReleaseIndex, runner: C) -> Self {
        Self {
            release_index,
            runner,
        }
    }
}

impl<'index, C: Check> SubCommand for Verify<'index, C> {
    type Output = ();

    /// Run the verifier against a Rust version which is obtained from the config.
    fn run(&self, config: &Config, reporter: &impl Reporter) -> TResult<Self::Output> {
        let rust_version = RustVersion::try_from_config(config)?;

        verify_msrv(
            reporter,
            config,
            self.release_index,
            rust_version,
            &self.runner,
        )?;

        Ok(())
    }
}

/// Verify whether a Cargo project is compatible with a `rustup run` command,
/// for the (given or specified) `rust_version`.
fn verify_msrv(
    reporter: &impl Reporter,
    config: &Config,
    release_index: &ReleaseIndex,
    rust_version: RustVersion,
    runner: &impl Check,
) -> TResult<()> {
    let bare_version = rust_version.version();
    let version =
        bare_version.try_to_semver(release_index.releases().iter().map(Release::version))?;

    let toolchain = ToolchainSpec::new(version, config.target());

    match runner.check(config, &toolchain)? {
        Outcome::Success(_) => success(reporter, toolchain),
        Outcome::Failure(_) if config.no_check_feedback() => {
            failure(reporter, toolchain, rust_version, None)
        }
        Outcome::Failure(f) => failure(reporter, toolchain, rust_version, Some(f.error_message)),
    }
}

// Report the successful verification to the user
fn success(reporter: &impl Reporter, toolchain: ToolchainSpec) -> TResult<()> {
    reporter.report_event(VerifyResult::compatible(toolchain))?;
    Ok(())
}

// Report the failed verification to the user, and return a VerifyFailed error
fn failure(
    reporter: &impl Reporter,
    toolchain: ToolchainSpec,
    rust_version: RustVersion,
    error: Option<String>,
) -> TResult<()> {
    reporter.report_event(VerifyResult::incompatible(toolchain, error))?;

    Err(CargoMSRVError::SubCommandVerify(Error::VerifyFailed(
        VerifyFailed::from(rust_version),
    )))
}

/// Error which can be returned if the verifier deemed the tested Rust version incompatible.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Crate source was found to be incompatible with Rust version '{}' specified {}", .0.rust_version, .0.source
    )]
    VerifyFailed(VerifyFailed),
}

/// Data structure which contains information about which version failed to verify, and where
/// we obtained this version from.
///
/// It is combination of the Rust version which was tested for compatibility and the source which was
/// used to find this tested Rust version.
#[derive(Debug)]
pub struct VerifyFailed {
    rust_version: BareVersion,
    source: RustVersionSource,
}

impl From<RustVersion> for VerifyFailed {
    fn from(value: RustVersion) -> Self {
        VerifyFailed {
            rust_version: value.rust_version,
            source: value.source,
        }
    }
}

/// A combination of a bare (two- or three component) Rust version and the source which was used to
/// locate this version.
#[derive(Debug)]
struct RustVersion {
    rust_version: BareVersion,
    source: RustVersionSource,
}

impl RustVersion {
    /// Obtain the rust-version from one of two sources, in order:
    /// 1. the rust-version given to the verify subcommand, or
    /// 2. the rust-version as specified in the Cargo manifest
    fn try_from_config(config: &Config) -> TResult<Self> {
        let rust_version = config.sub_command_config().verify().rust_version.as_ref();

        let (rust_version, source) = match rust_version {
            Some(v) => Ok((v.clone(), RustVersionSource::Arg)),
            None => {
                let path = config.context().manifest_path()?;
                let document = TomlDocumentReader::read_document(path)?;
                let manifest = CargoManifest::try_from(document)?;

                manifest
                    .minimum_rust_version()
                    .ok_or_else(|| CargoMSRVError::NoMSRVKeyInCargoToml(path.to_path_buf()))
                    .map(|v| (v.clone(), RustVersionSource::Manifest(path.to_path_buf())))
            }
        }?;

        Ok(Self {
            rust_version,
            source,
        })
    }

    /// Get the bare (two- or three component) version specifying the Rust version.
    fn version(&self) -> &BareVersion {
        &self.rust_version
    }
}

/// Source used to obtain a Rust version for the verifier.
#[derive(Debug, thiserror::Error)]
enum RustVersionSource {
    #[error("as --rust-version argument")]
    Arg,

    #[error("as MSRV in the Cargo manifest located at '{0}'")]
    Manifest(PathBuf),
}
