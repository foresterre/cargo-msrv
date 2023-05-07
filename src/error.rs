use owo_colors::OwoColorize;
use std::env;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::cli::rust_releases_opts::{ParseEditionError, ParseEditionOrVersionError};
use crate::log_level::ParseLogLevelError;
use crate::manifest::bare_version::{BareVersion, NoVersionMatchesManifestMsrvError};
use crate::manifest::reader::ManifestReaderError;
use crate::manifest::ManifestParseError;
use rust_releases::Release;

use crate::sub_command::{show, verify};

pub(crate) type TResult<T> = Result<T, CargoMSRVError>;

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

    #[error(transparent)]
    Io(#[from] IoError),

    #[error("{0}")]
    InvalidConfig(String),

    #[error(transparent)]
    InvalidRustVersionNumber(#[from] std::num::ParseIntError),

    #[error(transparent)]
    InvalidMsrvSet(#[from] InvalidMsrvSetError),

    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    #[error(transparent)]
    ManifestParseError(#[from] ManifestParseError),

    #[error(transparent)]
    ManifestReaderError(#[from] ManifestReaderError),

    #[error("No crate root found for given crate")]
    NoCrateRootFound,

    #[error(transparent)]
    NoToolchainsToTry(#[from] NoToolchainsToTryError),

    #[error("Unable to set MSRV for workspace, try setting it for individual packages instead.")]
    WorkspaceFound,

    #[error(transparent)]
    NoVersionMatchesManifestMSRV(#[from] NoVersionMatchesManifestMsrvError),

    #[error("Unable to find key 'package.rust-version' (or 'package.metadata.msrv') in '{0}'")]
    NoMSRVKeyInCargoToml(PathBuf),

    #[error(transparent)]
    ParseEdition(#[from] ParseEditionError),

    #[error(transparent)]
    ParseEditionOrVersion(#[from] ParseEditionOrVersionError),

    #[error(transparent)]
    ParseLogLevel(#[from] ParseLogLevelError),

    #[error("Unable to parse Cargo.toml: {0}")]
    ParseToml(#[from] toml_edit::TomlError),

    #[error(transparent)]
    RustReleasesSource(#[from] rust_releases::RustChangelogError),

    #[error(transparent)]
    #[cfg(feature = "rust-releases-dist-source")]
    RustReleasesRustDistSource(#[from] rust_releases::RustDistError),

    #[error("Unable to parse rust-releases source from '{0}'")]
    RustReleasesSourceParseError(String),

    #[error("There are no Rust releases in the rust-releases index")]
    RustReleasesEmptyReleaseSet,

    #[error(transparent)]
    RustupInstallFailed(#[from] RustupInstallFailed),

    #[error("Check toolchain (with `rustup run <toolchain> <command>`) failed.")]
    RustupRunWithCommandFailed,

    #[error(transparent)]
    SemverError(#[from] rust_releases::semver::Error),

    #[error(transparent)]
    SetMsrv(#[from] SetMsrvError),

    #[error("Unable to print event output")]
    Storyteller,

    #[error(transparent)]
    SubCommandVerify(#[from] verify::Error),

    #[error(transparent)]
    SubCommandShow(#[from] show::Error),

    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("The given toolchain could not be found. Run `rustup toolchain list` for an overview of installed toolchains.")]
    ToolchainNotInstalled,

    #[error("The given target could not be found. Run `rustup target list` for an overview of available toolchains.")]
    UnknownTarget,

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

    #[error("Unable to parse the CLI arguments. Use `cargo msrv help` for more info.")]
    UnableToParseCliArgs,

    #[error("The Rust stable version could not be parsed from the stable channel manifest.")]
    UnableToParseRustVersion,

    #[error("Unable to run the checking command. If --check <cmd> is specified, you could try to verify if you can run the cmd manually.")]
    UnableToRunCheck,

    #[error(transparent)]
    Path(#[from] PathError),
}

impl From<String> for CargoMSRVError {
    fn from(s: String) -> Self {
        Self::GenericMessage(s)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("IO error: '{error}'. caused by: '{source}'.")]
pub struct IoError {
    pub error: io::Error,
    pub source: IoErrorSource,
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

#[derive(Debug, thiserror::Error)]
pub enum SetMsrvError {
    #[error(
        "Unable to set the MSRV in the 'package.metadata' table: 'package.metadata' is not a table"
    )]
    NotATable,
}

#[derive(Debug, thiserror::Error)]
#[error("No Rust releases to check {} {} (search space: [{}])",
    min.as_ref().map(|s| format!("(min: {})", s)).unwrap_or_default(),
    max.as_ref().map(|s| format!("(max: {})", s)).unwrap_or_default(),
    search_space.iter().map(|r| r.version().to_string()).collect::<Vec<_>>().join(", ") )
]
pub struct NoToolchainsToTryError {
    pub(crate) min: Option<BareVersion>,
    pub(crate) max: Option<BareVersion>,
    pub(crate) search_space: Vec<Release>,
}

#[derive(Debug, thiserror::Error)]
#[error("No Rust releases match input '{}' (search space: [{}])",
    input,
    search_space.iter().map(|r| r.version().to_string()).collect::<Vec<_>>().join(", ") )
]
pub struct InvalidMsrvSetError {
    pub(crate) input: BareVersion,
    pub(crate) search_space: Vec<Release>,
}

impl<T> From<storyteller::EventReporterError<T>> for CargoMSRVError {
    fn from(_: storyteller::EventReporterError<T>) -> Self {
        CargoMSRVError::Storyteller
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to install toolchain '{}', rustup reported:\n    {}",
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupInstallFailed {
    pub(crate) toolchain_spec: String,
    pub(crate) stderr: String,
}

impl RustupInstallFailed {
    pub fn new(toolchain_spec: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            toolchain_spec: toolchain_spec.into(),
            stderr: stderr.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("No parent directory for '{}'", .0.display())]
    NoParent(PathBuf),

    #[error(transparent)]
    InvalidUtf8(#[from] InvalidUtf8Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct InvalidUtf8Error {
    error: Utf8PathErrorInner,
}

impl From<camino::FromPathError> for InvalidUtf8Error {
    fn from(value: camino::FromPathError) -> Self {
        Self {
            error: Utf8PathErrorInner::FromPath(value),
        }
    }
}

impl From<camino::FromPathBufError> for InvalidUtf8Error {
    fn from(value: camino::FromPathBufError) -> Self {
        Self {
            error: Utf8PathErrorInner::FromPathBuf(value),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Utf8PathErrorInner {
    #[error("Path contains non UTF-8 characters")]
    FromPath(camino::FromPathError),
    #[error("Path contains non UTF-8 characters (path: '{}')", .0.as_path().display())]
    FromPathBuf(camino::FromPathBufError),
}
