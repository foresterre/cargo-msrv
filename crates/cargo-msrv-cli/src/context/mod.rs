//! The conversion of the user input (opts) into the resolved configuration (context).

use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use crate::cli::{CargoMsrvOpts, SubCommand};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_msrv_context::context::error::{
    Error, InvalidUtf8Error, IoError, IoErrorSource, PathError, TResult,
};
use cargo_msrv_context::default_target::default_target;
#[cfg(any(
    feature = "rust-releases-changelog-source",
    feature = "rust-releases-github-source",
    feature = "rust-releases-dist-source"
))]
use cargo_msrv_context::types::BundledFallback;
use cargo_msrv_context::{
    CargoProject, CheckCommandContext, Context, EnvironmentContext, FindContext, ListContext,
    Package, Project, RustReleasesContext, SetContext, ShowContext, ToolchainContext,
    VerifyContext,
};
use std::convert::{TryFrom, TryInto};
use std::env;
use std::path::Path;

mod find;
mod list;
mod set;
mod show;
mod verify;

impl TryFrom<CargoMsrvOpts> for Context {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
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

impl From<RustReleasesOpts> for RustReleasesContext {
    fn from(opts: RustReleasesOpts) -> Self {
        Self {
            minimum_rust_version: opts.min.map(|min| min.as_bare_version()),
            maximum_rust_version: opts.max,
            consider_patch_releases: opts.include_all_patch_releases,
            release_source: opts.release_source,
            #[cfg(any(
                feature = "rust-releases-changelog-source",
                feature = "rust-releases-github-source",
                feature = "rust-releases-dist-source"
            ))]
            bundled_fallback: BundledFallback {
                max_age: opts.bundled_max_age,
                source: opts.bundled_fallback_source,
            },
        }
    }
}

impl TryFrom<ToolchainOpts> for ToolchainContext {
    type Error = Error;

    fn try_from(opts: ToolchainOpts) -> TResult<Self> {
        let host: &'static str = String::leak(default_target()?);

        let target: &'static str = match opts.target {
            Some(target) => String::leak(target),
            None => host,
        };

        let components: &'static [&'static str] = Vec::leak(
            opts.component
                .into_iter()
                .map(|s| {
                    let s: &'static str = String::leak(s);
                    s
                })
                .collect(),
        );

        Ok(Self {
            host,
            target,
            components,
        })
    }
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

impl<'shared_opts> TryFrom<&'shared_opts SharedOpts> for EnvironmentContext {
    type Error = Error;

    fn try_from(opts: &'shared_opts SharedOpts) -> TResult<Self> {
        let path = if let Some(path) = opts.path.as_ref() {
            // Use `--path` if specified. This is the oldest supported option.
            // This option refers to the root of a crate.
            Ok(path.clone())
        } else if let Some(path) = opts.manifest_path.as_ref() {
            // Use `--manifest-path` if specified. This was added later, and can not be specified
            // together with `--path`. This option refers to the `Cargo.toml` document
            // of a crate ("manifest").
            dunce::canonicalize(path)
                .map_err(|_| Error::Path(PathError::DoesNotExist(path.to_path_buf())))
                .and_then(|p| {
                    p.parent()
                        .map(Path::to_path_buf)
                        .ok_or_else(|| Error::Path(PathError::NoParent(path.to_path_buf())))
                })
        } else {
            // Otherwise, fall back to the current directory.
            env::current_dir().map_err(|error| {
                Error::Io(IoError {
                    error,
                    source: IoErrorSource::CurrentDir,
                })
            })
        }?;

        let root_crate_path: Utf8PathBuf = path
            .try_into()
            .map_err(|err| Error::Path(PathError::InvalidUtf8(InvalidUtf8Error::from(err))))?;

        let project = resolve_project(opts, &root_crate_path)?;

        Ok(Self {
            root_crate_path,
            project,
        })
    }
}

fn resolve_project(opts: &SharedOpts, root: &Utf8Path) -> TResult<Project> {
    let manifest_path = root.join("Cargo.toml");

    let is_cargo_project = manifest_path.try_exists().map_err(|error| IoError {
        error,
        source: IoErrorSource::ReadFile(manifest_path.clone()),
    })?;

    if !is_cargo_project {
        tracing::info!(
            action = "detect_cargo_workspace_packages",
            method = "cargo_metadata",
            cargo_project = false,
        );

        return match cargo_flag(opts) {
            Some(flag) => Err(Error::CargoFlagWithoutCargoProject {
                flag,
                root: root.to_path_buf(),
            }),
            None => Ok(Project::Bare),
        };
    }

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .map_err(|source| Error::CargoMetadata {
            path: manifest_path.clone(),
            source,
        })?;

    let members = metadata.workspace_packages();

    if let Some(unknown) = opts.workspace.package.iter().find(|name| {
        !members
            .iter()
            .any(|member| member.name.as_str() == name.as_str())
    }) {
        return Err(Error::UnknownPackage(unknown.clone()));
    }

    let (selected, excluded) = opts.workspace.to_clap_cargo().partition_packages(&metadata);

    tracing::info!(
        action = "detect_cargo_workspace_packages",
        method = "cargo_metadata",
        cargo_project = true,
        selected = ?selected.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        excluded = ?excluded.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
    );

    let packages = selected
        .into_iter()
        .map(Package::try_from)
        .collect::<TResult<Vec<_>>>()?;

    let project = CargoProject::new(metadata.workspace_root.clone(), packages)?;

    Ok(Project::Cargo(project))
}

fn cargo_flag(opts: &SharedOpts) -> Option<&'static str> {
    let workspace = &opts.workspace;

    [
        (workspace.workspace, "--workspace"),
        (workspace.all, "--all"),
        (!workspace.package.is_empty(), "--package"),
        (!workspace.exclude.is_empty(), "--exclude"),
        (opts.manifest_path.is_some(), "--manifest-path"),
    ]
    .into_iter()
    .find_map(|(used, flag)| used.then_some(flag))
}
