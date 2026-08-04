use camino::Utf8PathBuf;
use owo_colors::OwoColorize;
use std::env;
use std::string::FromUtf8Error;

use crate::cli::rust_releases_opts::ParseEditionOrVersionError;
use crate::manifest::ManifestParseError;
use cargo_msrv_context::types::{
    ParseEditionError, ParseListMsrvVariantError, ParseLogLevelError, ParseOutputFormatError,
    ParseReleaseSourceError, ParseTracingTargetOptionError,
};
use cargo_msrv_types::{BareVersion, NoVersionMatchesManifestMsrvError};
use rust_releases::Release;

pub use cargo_msrv_context::context::error::{
    Error as ContextError, InvalidUtf8Error, IoError, IoErrorSource, PathError,
};

use crate::sub_command::{show, verify};

pub(crate) type TResult<T> = Result<T, CargoMSRVError>;

#[derive(Debug, thiserror::Error)]
pub enum CargoMSRVError {
    #[error("Unable to parse minimum rust version: {0}")]
    BareVersionParse(#[from] cargo_msrv_types::bare_version::Error),

    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error(transparent)]
    Context(#[from] ContextError),

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
    LockfileHandler(#[from] LockfileHandlerError),

    #[error(transparent)]
    ManifestParseError(#[from] ManifestParseError),

    #[error("No crate root found for given crate")]
    NoCrateRootFound,

    #[error(transparent)]
    NoToolchainsToTry(#[from] NoToolchainsToTryError),

    #[error("Unable to set MSRV for workspace, try setting it for individual packages instead.")]
    WorkspaceFound,

    #[error(transparent)]
    NoVersionMatchesManifestMSRV(#[from] NoVersionMatchesManifestMsrvError),

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
    RustupError(#[from] RustupError),

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

    #[error(
        "The given toolchain could not be found. Run `rustup toolchain list` for an overview of installed toolchains."
    )]
    ToolchainNotInstalled,

    #[error(
        "The given target could not be found. Run `rustup target list` for an overview of available toolchains."
    )]
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

    #[error("Unable to run the check command: '{}' at '{}'", &command, &cwd)]
    UnableToRunCheck { command: String, cwd: Utf8PathBuf },

    #[error(transparent)]
    Path(#[from] PathError),
}

impl CargoMSRVError {
    pub fn should_highlight(&self) -> bool {
        matches!(
            self,
            Self::UnableToFindAnyGoodVersion { .. } | Self::InvalidMsrvSet(_)
        )
    }
}

impl From<String> for CargoMSRVError {
    fn from(s: String) -> Self {
        Self::GenericMessage(s)
    }
}

// The values of the command line options are parsed by the `cargo-msrv-cli` crate, which has its
// own, self contained errors. The conversions below keep these errors reportable as a
// `CargoMSRVError`.

impl From<ParseListMsrvVariantError> for CargoMSRVError {
    fn from(error: ParseListMsrvVariantError) -> Self {
        Self::InvalidConfig(error.to_string())
    }
}

impl From<ParseOutputFormatError> for CargoMSRVError {
    fn from(error: ParseOutputFormatError) -> Self {
        Self::InvalidConfig(error.to_string())
    }
}

impl From<ParseReleaseSourceError> for CargoMSRVError {
    fn from(error: ParseReleaseSourceError) -> Self {
        Self::RustReleasesSourceParseError(error.0)
    }
}

impl From<ParseTracingTargetOptionError> for CargoMSRVError {
    fn from(error: ParseTracingTargetOptionError) -> Self {
        Self::InvalidConfig(error.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SetMsrvError {
    #[error(
        "Unable to set the MSRV in the 'package.metadata' table: 'package.metadata' is not a table"
    )]
    NotATable,
}

#[derive(Debug, thiserror::Error)]
#[error("No Rust releases to check: the filtered search space is empty.{}",
    inner.as_ref().map(|clues| format!("{}", clues)).unwrap_or_default(),
)]
pub struct NoToolchainsToTryError {
    inner: Option<NoToolchainToTryClues>,
}

impl NoToolchainsToTryError {
    pub fn new_empty() -> Self {
        Self { inner: None }
    }

    pub fn with_clues(user_min: Option<BareVersion>, user_max: Option<BareVersion>) -> Self {
        Self {
            inner: Some(NoToolchainToTryClues {
                min: user_min,
                max: user_max,
            }),
        }
    }

    pub fn has_clues(&self) -> bool {
        self.inner.is_some()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(" Search space limited by user to min Rust '{}', and max Rust '{}'",
    min.as_ref().map(|s| format!("{}", s)).unwrap_or_else(|| "<not overridden>".to_string()),
    max.as_ref().map(|s| format!("{}", s)).unwrap_or_else(|| "<not overridden>".to_string()),
)]
pub struct NoToolchainToTryClues {
    min: Option<BareVersion>,
    max: Option<BareVersion>,
}

#[derive(Debug, thiserror::Error)]
#[error("No Rust releases match input '{}' (search space: [{}])",
    input,
    search_space.iter().map(|r| r.version().to_string()).collect::<Vec<_>>().join(", "))
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
#[error(transparent)]
pub enum RustupError {
    Install(#[from] RustupInstallError),
    AddComponent(#[from] RustupAddComponentError),
    AddTarget(#[from] RustupAddTargetError),
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to install toolchain '{}', rustup reported:\n    {}",
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupInstallError {
    pub toolchain_spec: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to add components '{}' to toolchain '{}', rustup reported:\n    {}",
    components,
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupAddComponentError {
    pub components: String,
    pub toolchain_spec: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Unable to add target '{}' to toolchain '{}', rustup reported:\n    {}",
    targets,
    toolchain_spec,
    stderr.trim_end().lines().collect::<Vec<_>>().join("\n    ").dimmed()
)]
pub struct RustupAddTargetError {
    pub targets: String,
    pub toolchain_spec: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Unable to set cleanup handler for lockfile")]
pub struct LockfileHandlerError;
