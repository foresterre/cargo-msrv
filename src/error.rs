use camino::Utf8PathBuf;
use std::env;
use std::string::FromUtf8Error;

use crate::cli::rust_releases_opts::ParseEditionOrVersionError;
use cargo_msrv_context::types::{
    ParseEditionError, ParseListMsrvVariantError, ParseLogLevelError, ParseOutputFormatError,
    ParseReleaseSourceError, ParseTracingTargetOptionError,
};
use cargo_msrv_rust_releases::{FetchIndexError, RustRelease, Stable};
use cargo_msrv_types::{BareVersion, NoVersionMatchesManifestMsrvError};

pub use cargo_msrv_context::context::error::{
    Error as ContextError, InvalidUtf8Error, IoError, IoErrorSource, PathError,
};
use cargo_msrv_rust_tools::{CargoMetadataResolveError, ManifestParseError};
pub use cargo_msrv_search::error::{
    LockfileHandlerError, NoToolchainsToTryError, RustupAddComponentError, RustupAddTargetError,
    RustupError, RustupInstallError,
};

use cargo_msrv_search::error::Error as SearchError;

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

    #[error(transparent)]
    FetchIndex(#[from] FetchIndexError),

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

    #[error("Unable to parse rust-releases source from '{0}'")]
    RustReleasesSourceParseError(String),

    #[error("There are no Rust releases in the rust-releases index")]
    RustReleasesEmptyReleaseSet,

    #[error(transparent)]
    RustupError(#[from] RustupError),

    #[error("Check toolchain (with `rustup run <toolchain> <command>`) failed.")]
    RustupRunWithCommandFailed,

    #[error(transparent)]
    SemverError(#[from] semver::Error),

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
#[error("No Rust releases match input '{}' (search space: [{}])",
    input,
    search_space.iter().map(|r| r.version().version.to_string()).collect::<Vec<_>>().join(", "))
]
pub struct InvalidMsrvSetError {
    pub(crate) input: BareVersion,
    pub(crate) search_space: Vec<RustRelease<Stable>>,
}

impl<T> From<storyteller::EventReporterError<T>> for CargoMSRVError {
    fn from(_: storyteller::EventReporterError<T>) -> Self {
        CargoMSRVError::Storyteller
    }
}

// The MSRV search and the toolchain compatibility check are provided by the `cargo-msrv-search`
// crate, which has its own, self contained errors. The conversion below keeps these errors
// reportable as a `CargoMSRVError`.

impl From<SearchError> for CargoMSRVError {
    fn from(error: SearchError) -> Self {
        match error {
            SearchError::Io(error) => Self::Io(error),
            SearchError::LockfileHandler(error) => Self::LockfileHandler(error),
            SearchError::NoToolchainsToTry(error) => Self::NoToolchainsToTry(error),
            SearchError::Rustup(error) => Self::RustupError(error),
            SearchError::Storyteller => Self::Storyteller,
            SearchError::UnableToRunCheck { command, cwd } => {
                Self::UnableToRunCheck { command, cwd }
            }
        }
    }
}

impl From<CargoMetadataResolveError> for CargoMSRVError {
    fn from(error: CargoMetadataResolveError) -> Self {
        match error {
            CargoMetadataResolveError::CargoMetadata(error) => Self::CargoMetadata(error),
            CargoMetadataResolveError::NoCrateRootFound => Self::NoCrateRootFound,
        }
    }
}
