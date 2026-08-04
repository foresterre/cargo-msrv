//! The conversion of the user input (opts) into the resolved configuration (context).

use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use crate::cli::{CargoMsrvOpts, SubCommand};
use camino::Utf8PathBuf;
use cargo_msrv_context::context::error::{
    Error, InvalidUtf8Error, IoError, IoErrorSource, PathError, TResult,
};
use cargo_msrv_context::default_target::default_target;
use cargo_msrv_context::{
    CheckCommandContext, Context, EnvironmentContext, FindContext, ListContext,
    RustReleasesContext, SetContext, ShowContext, ToolchainContext, VerifyContext,
    WorkspacePackages,
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
        }
    }
}

impl TryFrom<ToolchainOpts> for ToolchainContext {
    type Error = Error;

    fn try_from(opts: ToolchainOpts) -> TResult<Self> {
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

            tracing::info!(
                action = "detect_cargo_workspace_packages",
                method = "cargo_metadata",
                success = true,
                ?selected,
                ?excluded
            );

            WorkspacePackages::from_vec(selected)
        } else {
            tracing::info!(
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
