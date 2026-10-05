//! The `context` is the resolved configuration for the current run of `cargo-msrv`.
//!
//! The context is the synthesized user input (opts).
//! Where the user input deals with presentation, the context consists of only
//! the relevant data which is necessary for the functioning of the subcommand.
//!
//! Unlike the opts, the context is top down, not bottom up.

use crate::context::error::{Error, TResult};
#[cfg(any(
    feature = "rust-releases-changelog-source",
    feature = "rust-releases-github-source",
    feature = "rust-releases-dist-source"
))]
use crate::types::BundledFallback;
use crate::types::{Edition, LogLevel, ReleaseSource, TracingTargetOption};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_msrv_rust_tools::CargoManifest;
use version_number::Version;

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
    pub minimum_rust_version: Option<Version>,

    /// The maximum Rust version to consider (inclusive).
    pub maximum_rust_version: Option<Version>,

    /// Whether to consider patch releases as separate versions.
    pub consider_patch_releases: bool,

    /// The release source to use.
    pub release_source: ReleaseSource,

    #[cfg(any(
        feature = "rust-releases-changelog-source",
        feature = "rust-releases-github-source",
        feature = "rust-releases-dist-source"
    ))]
    pub bundled_fallback: BundledFallback,
}

impl RustReleasesContext {
    pub fn resolve_minimum_version(&self, package: Option<&Package>) -> Option<Version> {
        self.minimum_rust_version
            .clone()
            .or_else(|| package.map(|p| p.edition.as_version()))
    }
}

#[derive(Debug)]
pub struct ToolchainContext {
    /// The platform on which the toolchain is installed and run
    pub host: &'static str,

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
    /// The path to the root of a crate.
    ///
    /// Does not include a manifest file like Cargo.toml, so it's easy to append
    /// a file path like `Cargo.toml` or `Cargo.lock`.
    pub root_crate_path: Utf8PathBuf,

    pub project: Project,
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

    /// `None` for a bare project, which has no lockfile.
    pub fn lock(&self) -> Option<Utf8PathBuf> {
        match &self.project {
            Project::Cargo(p) => Some(p.workspace_root.join("Cargo.lock")),
            Project::Bare => None,
        }
    }

    pub fn workspace_root(&self) -> &Utf8Path {
        match &self.project {
            Project::Cargo(p) => &p.workspace_root,
            Project::Bare => &self.root_crate_path,
        }
    }

    pub fn packages_to_check(&self, check_cmd: &CheckCommandContext) -> Vec<Option<&Package>> {
        match &self.project {
            Project::Cargo(p) if check_cmd.rustup_command.is_none() => {
                p.packages().map(Some).collect()
            }
            Project::Cargo(_) | Project::Bare => vec![None],
        }
    }

    pub fn oldest_edition_package(&self) -> Option<&Package> {
        match &self.project {
            Project::Cargo(p) => p.packages().min_by_key(|p| p.edition),
            Project::Bare => None,
        }
    }

    pub fn selected_packages(&self) -> Option<Vec<SelectedPackage>> {
        match &self.project {
            Project::Cargo(p) => Some(p.packages().map(SelectedPackage::from).collect()),
            Project::Bare => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Project {
    /// No `Cargo.toml` at the root; only a custom check command can be used.
    Bare,
    Cargo(CargoProject),
}

#[derive(Clone, Debug)]
pub struct CargoProject {
    /// `metadata.workspace_root`, home of the shared `Cargo.lock` and toolchain file.
    pub workspace_root: Utf8PathBuf,
    /// The first package is kept apart, so the list can never be empty.
    pub first: Package,
    pub rest: Vec<Package>,
}

impl CargoProject {
    pub fn new(workspace_root: Utf8PathBuf, packages: Vec<Package>) -> TResult<Self> {
        let mut packages = packages.into_iter();
        let first = packages.next().ok_or(Error::EmptySelection)?;

        Ok(Self {
            workspace_root,
            first,
            rest: packages.collect(),
        })
    }

    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        std::iter::once(&self.first).chain(self.rest.iter())
    }
}

#[derive(Clone, Debug)]
pub struct Package {
    pub name: String,
    pub manifest_path: Utf8PathBuf,
    /// Already resolved by cargo, so `rust-version.workspace = true` works.
    pub rust_version: Option<Version>,
    pub edition: Edition,
}

impl TryFrom<&cargo_metadata::Package> for Package {
    type Error = Error;

    fn try_from(package: &cargo_metadata::Package) -> TResult<Self> {
        let manifest = CargoManifest::for_package(package)?;

        Ok(Self {
            name: package.name.to_string(),
            manifest_path: package.manifest_path.clone(),
            rust_version: manifest.minimum_rust_version().cloned(),
            edition: package.edition.as_str().parse()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectedPackage {
    pub name: String,
    pub path: Utf8PathBuf,
}

impl From<&Package> for SelectedPackage {
    fn from(package: &Package) -> Self {
        Self {
            name: package.name.clone(),
            path: package.manifest_path.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn package(edition: Edition) -> Package {
        Package {
            name: "a".to_string(),
            manifest_path: Utf8PathBuf::from("a/Cargo.toml"),
            rust_version: None,
            edition,
        }
    }

    #[test]
    fn minimum_version_without_package() {
        let min = RustReleasesContext::default().resolve_minimum_version(None);

        assert!(min.is_none());
    }

    #[test]
    fn minimum_version_from_package_edition() {
        let min = RustReleasesContext::default()
            .resolve_minimum_version(Some(&package(Edition::Edition2021)));

        assert_eq!(min, Some(Version::new_full_version(1, 56, 0)));
    }

    #[test]
    fn minimum_version_given_wins_from_edition() {
        let ctx = RustReleasesContext {
            minimum_rust_version: Some(Version::new_base_version(1, 40)),
            ..RustReleasesContext::default()
        };

        let min = ctx.resolve_minimum_version(Some(&package(Edition::Edition2021)));

        assert_eq!(min, Some(Version::new_base_version(1, 40)));
    }

    #[test]
    fn empty_selection() {
        let result = CargoProject::new(Utf8PathBuf::from("ws"), vec![]);

        assert!(matches!(result, Err(Error::EmptySelection)));
    }

    #[test]
    fn packages_in_selection_order() {
        let mut b = package(Edition::Edition2018);
        b.name = "b".to_string();

        let project = CargoProject::new(
            Utf8PathBuf::from("ws"),
            vec![package(Edition::Edition2021), b],
        )
        .unwrap();

        let names = project
            .packages()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["a", "b"]);
    }

    fn cargo_environment(packages: Vec<Package>) -> EnvironmentContext {
        EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("ws"),
            project: Project::Cargo(CargoProject::new(Utf8PathBuf::from("ws"), packages).unwrap()),
        }
    }

    fn check_cmd(custom: Option<Vec<String>>) -> CheckCommandContext {
        CheckCommandContext {
            cargo_features: None,
            cargo_all_features: false,
            cargo_no_default_features: false,
            rustup_command: custom,
        }
    }

    #[test]
    fn each_package_is_checked_with_the_default_command() {
        let mut b = package(Edition::Edition2018);
        b.name = "b".to_string();
        let env = cargo_environment(vec![package(Edition::Edition2021), b]);

        let names = env
            .packages_to_check(&check_cmd(None))
            .into_iter()
            .map(|p| p.map(|p| p.name.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(names, [Some("a"), Some("b")]);
    }

    #[test]
    fn custom_command_checks_the_project_once() {
        let env = cargo_environment(vec![package(Edition::Edition2021)]);

        let packages = env.packages_to_check(&check_cmd(Some(vec!["make".to_string()])));

        assert!(matches!(packages.as_slice(), [None]));
    }

    #[test]
    fn bare_project_is_checked_once() {
        let env = EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("ws"),
            project: Project::Bare,
        };

        assert!(matches!(
            env.packages_to_check(&check_cmd(None)).as_slice(),
            [None]
        ));
        assert!(env.lock().is_none());
        assert_eq!(env.workspace_root(), Utf8Path::new("ws"));
        assert!(env.selected_packages().is_none());
    }

    #[test]
    fn oldest_edition_package() {
        let mut b = package(Edition::Edition2018);
        b.name = "b".to_string();
        let env = cargo_environment(vec![package(Edition::Edition2021), b]);

        assert_eq!(env.oldest_edition_package().unwrap().name, "b");
    }
}
