//! The `context` is the resolved configuration for the current run of `cargo-msrv`.
//!
//! The context is the synthesized user input (opts).
//! Where the user input deals with presentation, the context consists of only
//! the relevant data which is necessary for the functioning of the subcommand.
//!
//! Unlike the opts, the context is top down, not bottom up.

use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::{SharedOpts, UserOutputOpts};
use crate::cli::toolchain_opts::ToolchainOpts;

use crate::error::{CargoMSRVError, InvalidUtf8Error, IoError, IoErrorSource, PathError};
use crate::manifest::bare_version::BareVersion;
use camino::{Utf8Path, Utf8PathBuf};
use clap::ValueEnum;
use std::convert::{TryFrom, TryInto};
use std::path::Path;
use std::str::FromStr;
use std::{env, fmt};

pub mod find;
pub mod list;
pub mod set;
pub mod show;
pub mod verify;

use crate::cli::custom_check_opts::CargoCheckOpts;
use crate::cli::rust_releases_opts::Edition;
use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::default_target::default_target;
use crate::log_level::LogLevel;
pub use find::FindContext;
pub use list::ListContext;
pub use set::SetContext;
pub use show::ShowContext;
pub use verify::VerifyContext;

/// Using sub-contexts allows us to write `From` implementations,
/// for each sub-command, where each only contains the relevant portion of
/// data.
#[derive(Debug)]
pub enum Context {
    Find(FindContext),
    List(ListContext),
    Set(SetContext),
    Show(ShowContext),
    Verify(VerifyContext),
}

impl Context {
    pub fn output_format(&self) -> OutputFormat {
        match self {
            Context::Find(ctx) => ctx.user_output.output_format,
            Context::List(ctx) => ctx.user_output.output_format,
            Context::Set(ctx) => ctx.user_output.output_format,
            Context::Show(ctx) => ctx.user_output.output_format,
            Context::Verify(ctx) => ctx.user_output.output_format,
        }
    }

    pub fn reporting_name(&self) -> &'static str {
        match self {
            Context::Find(_) => "find",
            Context::List(_) => "list",
            Context::Set(_) => "set",
            Context::Show(_) => "show",
            Context::Verify(_) => "verify",
        }
    }

    /// Returns the inner find context, if it was present.
    pub fn to_find_context(self) -> Option<FindContext> {
        if let Self::Find(ctx) = self {
            Some(ctx)
        } else {
            None
        }
    }

    /// Returns the inner find context, if it was present.
    pub fn to_verify_context(self) -> Option<VerifyContext> {
        if let Self::Verify(ctx) = self {
            Some(ctx)
        } else {
            None
        }
    }
}

impl TryFrom<CargoMsrvOpts> for Context {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let ctx = match opts.subcommand.as_ref() {
            None => Self::Find(FindContext::try_from(opts)?),
            Some(SubCommand::List(_)) => Self::List(ListContext::try_from(opts)?),
            Some(SubCommand::Set(_)) => Self::Set(SetContext::try_from(opts)?),
            Some(SubCommand::Show) => Self::Show(ShowContext::try_from(opts)?),
            Some(SubCommand::Verify(_)) => Self::Verify(VerifyContext::try_from(opts)?),
        };

        Ok(ctx)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RustReleasesContext {
    /// The minimum Rust version to consider.
    pub minimum_rust_version: Option<BareVersion>,

    /// The maximum Rust version to consider (inclusive).
    pub maximum_rust_version: Option<BareVersion>,

    /// Whether to consider patch releases as separate versions.
    pub consider_patch_releases: bool,

    /// The release source to use.
    pub release_source: ReleaseSource,
}

impl From<RustReleasesOpts> for RustReleasesContext {
    fn from(opts: RustReleasesOpts) -> Self {
        Self {
            minimum_rust_version: opts.min.map(|min| min.as_bare_version()),
            maximum_rust_version: opts.max,
            consider_patch_releases: opts.include_all_patch_releases,
            release_source: opts.release_source,
        }
    }
}

impl RustReleasesContext {
    // This is necessary because we need to fetch the minimum version possibly from the Cargo.toml
    // via the edition key; but where that file should be located is only after we have an
    // EnvironmentContext.
    pub fn resolve_minimum_version(
        &self,
        env: &EnvironmentContext,
    ) -> Result<Option<BareVersion>, CargoMSRVError> {
        // Precedence 1: Supplied values take precedence over all else.
        if let Some(min) = &self.minimum_rust_version {
            return Ok(Some(min.clone()));
        }

        // Precedence 2: Read from manifest
        let manifest = env.manifest();
        let contents = std::fs::read_to_string(&manifest).map_err(|error| IoError {
            error,
            source: IoErrorSource::ReadFile(manifest.clone()),
        })?;

        let document = contents
            .parse::<toml_edit::Document>()
            .map_err(CargoMSRVError::ParseToml)?;

        if let Some(edition) = document
            .as_table()
            .get("package")
            .and_then(toml_edit::Item::as_table)
            .and_then(|package_table| package_table.get("edition"))
            .and_then(toml_edit::Item::as_str)
        {
            let edition = edition.parse::<Edition>()?;

            return Ok(Some(edition.as_bare_version()));
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub struct ToolchainContext {
    /// The target of the toolchain
    pub target: String,
}

impl TryFrom<ToolchainOpts> for ToolchainContext {
    type Error = CargoMSRVError;

    fn try_from(opts: ToolchainOpts) -> Result<Self, Self::Error> {
        let target = if let Some(target) = opts.target {
            target
        } else {
            default_target()?
        };

        Ok(Self { target })
    }
}

#[derive(Debug)]
pub struct CargoCheckContext {
    pub cargo_features: Option<Vec<String>>,

    pub cargo_all_features: bool,

    pub cargo_no_default_features: bool,

    /// The custom `Rustup` command to invoke for a toolchain.
    pub rustup_command: Option<Vec<String>>,
}

impl From<CargoCheckOpts> for CargoCheckContext {
    fn from(opts: CargoCheckOpts) -> Self {
        Self {
            cargo_features: opts.features,
            cargo_all_features: opts.all_features,
            cargo_no_default_features: opts.no_default_features,
            rustup_command: opts.custom_check_command,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvironmentContext {
    /// The path to the root of a crate.
    ///
    /// Does not include a manifest file like Cargo.toml, so it's easy to append
    /// a file path like `Cargo.toml` or `Cargo.lock`.
    pub crate_path: Utf8PathBuf,
}

impl<'shared_opts> TryFrom<&'shared_opts SharedOpts> for EnvironmentContext {
    type Error = CargoMSRVError;

    fn try_from(opts: &'shared_opts SharedOpts) -> Result<Self, Self::Error> {
        let path = if let Some(path) = opts.path.as_ref() {
            // Use `--path` if specified. This is the oldest supported option.
            // This option refers to the root of a crate.
            Ok(path.clone())
        } else if let Some(path) = opts.manifest_path.as_ref() {
            // Use `--manifest-path` if specified. This was added later, and can not be specified
            // together with `--path`. This option refers to the `Cargo.toml` document
            // of a crate ("manifest").
            path.parent()
                .ok_or_else(|| CargoMSRVError::Path(PathError::NoParent(path.to_path_buf())))
                .map(Path::to_path_buf)
        } else {
            // Otherwise, fall back to the current directory.
            env::current_dir().map_err(|error| {
                CargoMSRVError::Io(IoError {
                    error,
                    source: IoErrorSource::CurrentDir,
                })
            })
        }?;

        let crate_path = path.try_into().map_err(|err| {
            CargoMSRVError::Path(PathError::InvalidUtf8(InvalidUtf8Error::from(err)))
        })?;

        Ok(Self { crate_path })
    }
}

impl EnvironmentContext {
    /// Path to the crate root
    pub fn root(&self) -> &Utf8Path {
        &self.crate_path
    }

    /// The path to the Cargo manifest
    pub fn manifest(&self) -> Utf8PathBuf {
        self.crate_path.join("Cargo.toml")
    }

    /// The path to the Cargo lock file
    pub fn lock(&self) -> Utf8PathBuf {
        self.crate_path.join("Cargo.lock")
    }
}

#[derive(Clone, Debug)]
pub struct UserOutputContext {
    /// The output format to use, or `None` if
    /// no user output should be presented to the user.
    pub output_format: OutputFormat,
}

impl From<UserOutputOpts> for UserOutputContext {
    fn from(opts: UserOutputOpts) -> Self {
        if opts.no_user_output {
            Self {
                output_format: OutputFormat::None,
            }
        } else {
            Self {
                output_format: opts.output_format,
            }
        }
    }
}

// ---

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Progress bar rendered to stderr
    #[default]
    Human,
    /// Json status updates printed to stdout
    Json,
    /// Minimal output, usually just the result, such as the MSRV or whether verify succeeded or failed
    Minimal,
    /// No output -- meant to be used for debugging and testing
    #[value(skip)]
    None,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Json => write!(f, "json"),
            Self::Minimal => write!(f, "minimal"),
            Self::None => write!(f, "none"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            "minimal" => Ok(Self::Minimal),
            unknown => Err(CargoMSRVError::InvalidConfig(format!(
                "Given output format '{}' is not valid",
                unknown
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseSource {
    #[default]
    RustChangelog,
    #[cfg(feature = "rust-releases-dist-source")]
    RustDist,
}

impl FromStr for ReleaseSource {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl From<ReleaseSource> for &'static str {
    fn from(value: ReleaseSource) -> Self {
        match value {
            ReleaseSource::RustChangelog => "rust-changelog",
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => "rust-dist",
        }
    }
}

impl TryFrom<&str> for ReleaseSource {
    type Error = CargoMSRVError;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
        match source {
            "rust-changelog" => Ok(Self::RustChangelog),
            #[cfg(feature = "rust-releases-dist-source")]
            "rust-dist" => Ok(Self::RustDist),
            s => Err(CargoMSRVError::RustReleasesSourceParseError(s.to_string())),
        }
    }
}

impl fmt::Display for ReleaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustChangelog => write!(f, "rust-changelog"),
            #[cfg(feature = "rust-releases-dist-source")]
            Self::RustDist => write!(f, "rust-dist"),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMethod {
    Linear,
    #[default]
    Bisect,
}

impl From<SearchMethod> for &'static str {
    fn from(method: SearchMethod) -> Self {
        match method {
            SearchMethod::Linear => "linear",
            SearchMethod::Bisect => "bisect",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TracingOptions {
    target: TracingTargetOption,
    level: LogLevel,
}

impl TracingOptions {
    pub fn new(target: TracingTargetOption, level: LogLevel) -> Self {
        Self { target, level }
    }
}

impl Default for TracingOptions {
    fn default() -> Self {
        Self {
            target: TracingTargetOption::File,
            level: LogLevel::default(),
        }
    }
}

impl TracingOptions {
    pub fn target(&self) -> &TracingTargetOption {
        &self.target
    }

    pub fn level(&self) -> &LogLevel {
        &self.level
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum TracingTargetOption {
    File,
    Stdout,
}

impl Default for TracingTargetOption {
    fn default() -> Self {
        Self::File
    }
}

impl TracingTargetOption {
    pub const FILE: &'static str = "file";
    pub const STDOUT: &'static str = "stdout";
}

impl FromStr for TracingTargetOption {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::FILE => Ok(Self::File),
            Self::STDOUT => Ok(Self::Stdout),
            unknown => Err(CargoMSRVError::InvalidConfig(format!(
                "Given log target '{}' is not valid",
                unknown
            ))),
        }
    }
}
