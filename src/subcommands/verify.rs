use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use rust_releases::{Release, ReleaseIndex};

use toml_edit::Document;

use crate::check::Check;
use crate::config::Config;
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::manifest::bare_version::BareVersion;
use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::outcome::Outcome;
use crate::reporter::Reporter;
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
    type Output = ();

    /// Run the verifier against a Rust version which is obtained from the config.
    fn run(&self, config: &Config, _reporter: &impl Reporter) -> TResult<Self::Output> {
        let rust_version = RustVersion::try_from_config(config)?;

        verify_msrv(config, self.release_index, rust_version, &self.runner)?;

        Ok(())
    }
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
    release_index: &ReleaseIndex,
    rust_version: RustVersion,
    runner: &impl Check,
) -> TResult<()> {
    let bare_version = rust_version.version();
    let version =
        bare_version.try_to_semver(release_index.releases().iter().map(Release::version))?;

    let toolchain = ToolchainSpec::new(version, config.target());

    match runner.check(config, &toolchain)? {
        Outcome::Success(_) => Ok(()),
        Outcome::Failure(_) => Err(CargoMSRVError::SubCommandVerify(Error::VerifyFailed(
            VerifyFailed::from(rust_version),
        ))),
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
                let path = config.ctx().manifest_path(config)?;
                let manifest = parse_manifest(path)?;

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
