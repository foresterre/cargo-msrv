use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use rust_releases::{semver, Release, ReleaseIndex};
use storyteller::Reporter;
use toml_edit::Document;

use crate::check::Check;
use crate::config::{Config, ModeIntent};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::manifest::bare_version::BareVersion;
use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::outcome::Outcome;
use crate::subcommands::SubCommand;
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
    /// Run the verifier against a Rust version which is obtained from the config.
    fn run(&self, config: &Config, reporter: &impl Reporter) -> TResult<()> {
        let rust_version = RustVersion::try_from_config(config)?;

        let result = verify_msrv(
            config,
            reporter,
            self.release_index,
            rust_version,
            &self.runner,
        );

        // report outcome
        report_result(result.as_ref(), config, reporter);

        result.map(|_| ())
    }
}

/// Report the outcome to the user
fn report_result(
    result: Result<&Outcome, &CargoMSRVError>,
    config: &Config,
    reporter: &impl Reporter,
) {
    match result.as_ref() {
        Ok(outcome) => {
            // todo!
            // reporter.finish_success(ModeIntent::Verify, Some(outcome.version()));
        }
        Err(CargoMSRVError::SubCommandVerify(Error::VerifyFailed { .. })) => {
            let cmd = config.check_command_string();
            // todo!
            // reporter.finish_failure(ModeIntent::Verify, Some(&cmd));
        }
        Err(_) => {}
    };
}

/// Parse the cargo manifest from the given path.
fn parse_manifest(path: &Path) -> TResult<CargoManifest> {
    let contents = std::fs::read_to_string(path).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(path.to_path_buf()),
    })?;

    let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    CargoManifest::try_from(manifest)
}

/// Verify whether a Cargo project is compatible with a `rustup run` command,
/// for the (given or specified) `rust_version`.
fn verify_msrv(
    config: &Config,
    reporter: &impl Reporter,
    release_index: &ReleaseIndex,
    rust_version: RustVersion,
    runner: &impl Check,
) -> TResult<Outcome> {
    // todo!
    // reporter.mode(ModeIntent::Verify);

    let bare_version = rust_version.version();
    let version =
        bare_version.try_to_semver(release_index.releases().iter().map(Release::version))?;

    let toolchain = ToolchainSpec::new(version, config.target());
    let outcome = runner.check(config, &toolchain)?;

    if outcome.is_success() {
        Ok(outcome)
    } else {
        Err(CargoMSRVError::SubCommandVerify(Error::VerifyFailed(
            VerifyFailed {
                rust_version: version.clone(),
                source: rust_version.into_source(),
            },
        )))
    }
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
    rust_version: semver::Version,
    source: RustVersionSource,
}

/// A combination of a bare (two- or three component) Rust version and the source which was used to
/// locate this version.
struct RustVersion {
    version: BareVersion,
    source: RustVersionSource,
}

impl RustVersion {
    /// Obtain the rust-version from one of two sources, in order:
    /// 1. the rust-version given to the verify subcommand, or
    /// 2. the rust-version as specified in the Cargo manifest
    fn try_from_config(config: &Config) -> TResult<Self> {
        let rust_version = config.sub_command_config().verify().rust_version.as_ref();

        let (version, source) = match rust_version {
            Some(v) => Ok((v.clone(), RustVersionSource::Arg)),
            None => {
                let path = config.ctx().manifest_path(config)?;
                let manifest = parse_manifest(path)?;

                manifest
                    .minimum_rust_version()
                    .ok_or_else(|| CargoMSRVError::NoMSRVKeyInCargoToml(path.to_path_buf()))
                    .map(|v| (v.clone(), RustVersionSource::Manifest(path.to_path_buf())))
            }
        }?;

        Ok(Self { version, source })
    }

    /// Get the bare (two- or three component) version specifying the Rust version.
    fn version(&self) -> &BareVersion {
        &self.version
    }

    /// Consume the [`RustVersion`] and return its [`RustVersionSource`].
    fn into_source(self) -> RustVersionSource {
        self.source
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
