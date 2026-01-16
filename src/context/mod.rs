//! The `context` is the resolved configuration for the current run of `cargo-msrv`.
//!
//! The context is the synthesized user input (opts).
//! Where the user input deals with presentation, the context consists of only
//! the relevant data which is necessary for the functioning of the subcommand.
//!
//! Unlike the opts, the context is top down, not bottom up.

use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
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

use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::rust_releases_opts::Edition;
use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::log_level::LogLevel;
use crate::reporter::event::SelectedPackage;
use crate::rust::default_target::default_target;
pub use find::FindContext;
pub use list::ListContext;
pub use set::SetContext;
pub use show::ShowContext;
pub use verify::VerifyContext;

/// A `context` in `cargo-msrv`, is a definitive and flattened set of options,
/// required for the program (and its selected sub-command) to function.
///
/// Where various `[...]Opts` structs are used to present an interface to the user,
/// these contexts are used to present an interface to the program.
/// These `[...]Opts` structs commonly have a tree structure, whereas the contexts
/// are intended to be at most 1 level of indirection deep.
/// In addition, the `[...]Opts` structs are used to present a CLI interface
/// using `clap` as an argument parser, but may be just one way to provide user
/// input. Alternative user interfaces may be provided, such as one which parses
/// environment variables and another which reads inputs from a configuration
/// file. If multiple inputs are provided, they should be merged with a specified
/// precedence. The final, flattened result shall be used as the program's internal
/// interface, i.e. this `context`.
///
/// Using sub-contexts allows us to write `TryFrom` implementations,
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
    pub fn reporting_name(&self) -> &'static str {
        match self {
            Context::Find(_) => "find",
            Context::List(_) => "list",
            Context::Set(_) => "set",
            Context::Show(_) => "show",
            Context::Verify(_) => "verify",
        }
    }

    pub fn environment_context(&self) -> &EnvironmentContext {
        match self {
            Context::Find(ctx) => &ctx.environment,
            Context::List(ctx) => &ctx.environment,
            Context::Set(ctx) => &ctx.environment,
            Context::Show(ctx) => &ctx.environment,
            Context::Verify(ctx) => &ctx.environment,
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
        let ctx = match opts.subcommand {
            SubCommand::Find(_) => Self::Find(FindContext::try_from(opts)?),
            SubCommand::List(_) => Self::List(ListContext::try_from(opts)?),
            SubCommand::Set(_) => Self::Set(SetContext::try_from(opts)?),
            SubCommand::Show => Self::Show(ShowContext::try_from(opts)?),
            SubCommand::Verify(_) => Self::Verify(VerifyContext::try_from(opts)?),
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
            .parse::<toml_edit::DocumentMut>()
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
    pub target: &'static str,

    /// Components to be installed for the toolchain
    pub components: &'static [&'static str],
}

impl TryFrom<ToolchainOpts> for ToolchainContext {
    type Error = CargoMSRVError;

    fn try_from(opts: ToolchainOpts) -> Result<Self, Self::Error> {
        let target = if let Some(target) = opts.target {
            target
        } else {
            default_target()?
        };

        let target: &'static str = String::leak(target);

        let components: &'static [&'static str] = Vec::leak(
            opts.component
                .into_iter()
                .map(|s| {
                    let s: &'static str = String::leak(s);
                    s
                })
                .collect(),
        );

        Ok(Self { target, components })
    }
}

#[derive(Debug)]
pub struct CheckCommandContext {
    pub cargo_features: Option<Vec<String>>,

    pub cargo_all_features: bool,

    pub cargo_no_default_features: bool,

    /// The custom `Rustup` command to invoke for a toolchain.
    pub rustup_command: Option<Vec<String>>,
}

impl From<CustomCheckOpts> for CheckCommandContext {
    fn from(opts: CustomCheckOpts) -> Self {
        Self {
            cargo_features: opts.features,
            cargo_all_features: opts.all_features,
            cargo_no_default_features: opts.no_default_features,
            rustup_command: opts.custom_check_opts,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvironmentContext {
    // TODO: Some parts assume a Cargo crate, but that's not strictly a requirement
    //  of cargo-msrv (only rustup is). We should fix this.
    /// The path to the root of a crate.
    ///
    /// Does not include a manifest file like Cargo.toml, so it's easy to append
    /// a file path like `Cargo.toml` or `Cargo.lock`.
    pub root_crate_path: Utf8PathBuf,

    /// Resolved workspace
    pub workspace_packages: WorkspacePackages,
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
            dunce::canonicalize(path)
                .map_err(|_| CargoMSRVError::Path(PathError::DoesNotExist(path.to_path_buf())))
                .and_then(|p| {
                    p.parent().map(Path::to_path_buf).ok_or_else(|| {
                        CargoMSRVError::Path(PathError::NoParent(path.to_path_buf()))
                    })
                })
        } else {
            // Otherwise, fall back to the current directory.
            env::current_dir().map_err(|error| {
                CargoMSRVError::Io(IoError {
                    error,
                    source: IoErrorSource::CurrentDir,
                })
            })
        }?;

        let root_crate_path: Utf8PathBuf = path.try_into().map_err(|err| {
            CargoMSRVError::Path(PathError::InvalidUtf8(InvalidUtf8Error::from(err)))
        })?;

        // Only select packages if this is a Cargo project.
        // For now, to be pragmatic, we'll take a shortcut and say that it is so,
        // if the cargo metadata command succeeds. If it doesn't, we'll fall
        // back to just the default package.
        let workspace_packages = if let Ok(metadata) = cargo_metadata::MetadataCommand::new()
            .manifest_path(root_crate_path.join("Cargo.toml"))
            .exec()
        {
            let partition = opts.workspace.partition_packages(&metadata);
            let selected = partition.0.into_iter().cloned().collect();
            let excluded = partition.1;

            info!(
                action = "detect_cargo_workspace_packages",
                method = "cargo_metadata",
                success = true,
                ?selected,
                ?excluded
            );

            WorkspacePackages::from_vec(selected)
        } else {
            info!(
                action = "detect_cargo_workspace_packages",
                method = "cargo_metadata",
                success = false,
            );

            WorkspacePackages::default()
        };

        Ok(Self {
            root_crate_path,
            workspace_packages,
        })
    }
}

impl EnvironmentContext {
    /// Path to the crate root
    pub fn root(&self) -> &Utf8Path {
        &self.root_crate_path
    }

    /// The path to the Cargo manifest
    pub fn manifest(&self) -> Utf8PathBuf {
        self.root_crate_path.join("Cargo.toml")
    }

    /// The path to the Cargo lock file
    pub fn lock(&self) -> Utf8PathBuf {
        self.root_crate_path.join("Cargo.lock")
    }
}

// ---

#[derive(Clone, Debug, Default)]
pub struct WorkspacePackages {
    selected: Option<Vec<cargo_metadata::Package>>,
}

impl WorkspacePackages {
    pub fn from_vec(selected: Vec<cargo_metadata::Package>) -> Self {
        Self {
            selected: Some(selected),
        }
    }

    pub fn selected(&self) -> Option<Vec<SelectedPackage>> {
        self.selected.as_deref().map(|pks| {
            pks.iter()
                .map(|pkg| SelectedPackage {
                    name: pkg.name.to_string(),
                    path: pkg.manifest_path.to_path_buf(),
                })
                .collect()
        })
    }

    /// The default package is used when either:
    /// 1. No packages were selected (e.g. because we are not in a cargo workspace or do not use cargo)
    /// 2. No workspace flags like --workspace, --package, --all or --exclude are used
    ///
    /// See [clap_cargo::Workspace](https://docs.rs/clap-cargo/latest/clap_cargo/struct.Workspace.html) which is
    /// currently used for the selection.
    pub fn use_default_package(&self) -> bool {
        self.selected_packages().is_empty()
    }

    /// The slice of selected packages.
    /// If empty, either no workspace selection flag was used, or cargo_metadata failed,
    /// for example because it wasn't a cargo workspace.
    pub fn selected_packages(&self) -> &[cargo_metadata::Package] {
        self.selected.as_deref().unwrap_or_default()
    }
}

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

#[derive(Debug, Copy, Clone, ValueEnum, Default)]
pub enum TracingTargetOption {
    #[default]
    File,
    Stdout,
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
