use std::env;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::fetch::ToolchainSpecifier;
use crate::manifest::bare_version::NoVersionMatchesManifestMsrvError;
use crate::subcommands::verify;

pub type TResult<T> = Result<T, CargoMSRVError>;

#[derive(Debug, thiserror::Error)]
pub enum CargoMSRVError {
    #[error("Unable to parse minimum rust version: {0}")]
    BareVersionParse(#[from] crate::manifest::bare_version::Error),

    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("The default host triple (target) could not be found.")]
    DefaultHostTripleNotFound,

    #[error(transparent)]
    Env(#[from] env::VarError),

    #[error("{0}")]
    GenericMessage(String),

    #[error("IO error: '{error}'. caused by: '{source}'.")]
    Io {
        error: io::Error,
        source: IoErrorSource,
    },

    #[error("{0}")]
    InvalidConfig(String),

    #[error(transparent)]
    InvalidRustVersionNumber(#[from] std::num::ParseIntError),

    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    #[error("No crate root found for given crate")]
    NoCrateRootFound,

    #[error(transparent)]
    NoVersionMatchesManifestMSRV(#[from] NoVersionMatchesManifestMsrvError),

    #[error("Unable to find key 'package.rust-version' (or 'package.metadata.msrv') in '{0}'")]
    NoMSRVKeyInCargoToml(PathBuf),

    #[error("Unable to parse Cargo.toml: {0}")]
    ParseToml(#[from] toml_edit::TomlError),

    #[error(transparent)]
    RustReleasesSource(#[from] rust_releases::RustChangelogError),

    #[error(transparent)]
    RustReleasesRustDistSource(#[from] rust_releases::RustDistError),

    #[error("Unable to parse rust-releases source from '{0}'")]
    RustReleasesSourceParseError(String),

    #[error("Unable to install toolchain with `rustup install {0}`.")]
    RustupInstallFailed(ToolchainSpecifier),

    #[error("Check toolchain (with `rustup run <toolchain> <command>`) failed.")]
    RustupRunWithCommandFailed,

    #[error(transparent)]
    SemverError(#[from] rust_releases::semver::Error),

    #[error(transparent)]
    SubCommandVerify(#[from] verify::Error),

    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("The given toolchain could not be found. Run `rustup toolchain list` for an overview of installed toolchains.")]
    ToolchainNotInstalled,

    #[error("The given target could not be found. Run `rustup target list` for an overview of available toolchains.")]
    UnknownTarget,

    #[error("Unable to access log folder, run with --no-log to try again without logging.")]
    UnableToAccessLogFolder,

    #[error("Unable to get or store the channel manifest on disk.")]
    UnableToCacheChannelManifest,

    #[error(
        r#"Unable to find a Minimum Supported Rust Version (MSRV).

If you think this result is erroneous, please run: `{command}` manually.

If the above does succeed, or you think cargo-msrv errored in another way, please feel free to
report the issue at: https://github.com/foresterre/cargo-msrv/issues

Thank you in advance!"#
    )]
    UnableToFindAnyGoodVersion { command: String },

    #[error("Unable to init logger, run with --no-log to try again without logging.")]
    UnableToInitTracing,

    #[error("Unable to parse the CLI arguments. Use `cargo msrv help` for more info.")]
    UnableToParseCliArgs,

    #[error("The Rust stable version could not be parsed from the stable channel manifest.")]
    UnableToParseRustVersion,

    #[error("Unable to run the checking command. If --check <cmd> is specified, you could try to verify if you can run the cmd manually.")]
    UnableToRunCheck,
}

impl From<String> for CargoMSRVError {
    fn from(s: String) -> Self {
        Self::GenericMessage(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IoErrorSource {
    #[error("Unable to determine current working directory")]
    CurrentDir,

    #[error("Unable to open file '{0}'")]
    OpenFile(PathBuf),

    #[error("Unable to read file '{0}'")]
    ReadFile(PathBuf),

    #[error("Unable to write file '{0}'")]
    WriteFile(PathBuf),

    #[error("Unable to remove file '{0}'")]
    RemoveFile(PathBuf),

    #[error("Unable to rename file '{0}'")]
    RenameFile(PathBuf),

    #[error("Unable to spawn process '{0:?}'")]
    SpawnProcess(OsString),

    #[error("Unable to collect output from '{0:?}', or process did not terminate properly")]
    WaitForProcessAndCollectOutput(OsString),
}
