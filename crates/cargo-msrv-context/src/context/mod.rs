//! The `context` is the resolved configuration for the current run of `cargo-msrv`.
//!
//! The context is the synthesized user input (opts).
//! Where the user input deals with presentation, the context consists of only
//! the relevant data which is necessary for the functioning of the subcommand.
//!
//! Unlike the opts, the context is top down, not bottom up.

use crate::context::error::{Error, IoError, IoErrorSource, TResult};
use crate::types::{Edition, LogLevel, ReleaseSource, TracingTargetOption};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_msrv_types::BareVersion;

pub mod error;
pub mod find;
pub mod list;
pub mod set;
pub mod show;
pub mod verify;

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

impl RustReleasesContext {
    // This is necessary because we need to fetch the minimum version possibly from the Cargo.toml
    // via the edition key; but where that file should be located is only after we have an
    // EnvironmentContext.
    pub fn resolve_minimum_version(
        &self,
        env: &EnvironmentContext,
    ) -> TResult<Option<BareVersion>> {
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
            .map_err(Error::ParseToml)?;

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

#[derive(Debug)]
pub struct CheckCommandContext {
    pub cargo_features: Option<Vec<String>>,

    pub cargo_all_features: bool,

    pub cargo_no_default_features: bool,

    /// The custom `Rustup` command to invoke for a toolchain.
    pub rustup_command: Option<Vec<String>>,
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectedPackage {
    pub name: String,
    pub path: Utf8PathBuf,
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
