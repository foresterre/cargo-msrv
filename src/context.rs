//! The `context` is the resolved configuration for the current run of `cargo-msrv`.
//!
//! The context is the synthesized user input (opts).
//! Where the user input deals with presentation, the context consists of only
//! the relevant data which is necessary for the functioning of the subcommand.
//!
//! Unlike the opts, the context is top down, not bottom up.

use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::{DebugOutputOpts, SharedOpts, UserOutputOpts};
use crate::cli::toolchain_opts::ToolchainOpts;

use crate::config::{OutputFormat, ReleaseSource, TracingTargetOption};
use crate::context::find::FindContext;
use crate::context::list::ListContext;
use crate::context::set::SetContext;
use crate::context::show::ShowContext;
use crate::context::verify::VerifyContext;
use crate::error::{CargoMSRVError, InvalidUtf8Error, IoError, IoErrorSource, PathError};
use crate::log_level::LogLevel;
use crate::manifest::bare_version::BareVersion;
use camino::Utf8PathBuf;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::path::Path;

mod find;
mod list;
mod set;
mod show;
mod verify;

/// Using sub-contexts allows us to write `From` implementations,
/// for each sub-command, where each only contains the relevant portion of
/// data.
#[derive(Debug)]
pub enum Context {
    Find(FindContext),
    Verify(VerifyContext),
    List(ListContext),
    Set(SetContext),
    Show(ShowContext),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ToolchainContext {
    /// The target of the toolchain
    pub target: Option<String>,
}

impl From<ToolchainOpts> for ToolchainContext {
    fn from(opts: ToolchainOpts) -> Self {
        Self {
            target: opts.target,
        }
    }
}

#[derive(Debug)]
pub struct CustomCheckContext {
    /// The custom `Rustup` command to invoke for a toolchain.
    pub custom_rustup_command: Vec<String>,
}

impl From<CustomCheckOpts> for CustomCheckContext {
    fn from(opts: CustomCheckOpts) -> Self {
        Self {
            custom_rustup_command: opts.custom_check_command,
        }
    }
}

#[derive(Debug)]
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
            /// Use `--path` if specified. This is the oldest supported option.
            /// This option refers to the root of a crate.
            Ok(path.clone())
        } else if let Some(path) = opts.manifest_path.as_ref() {
            /// Use `--manifest-path` if specified. This was added later, and can not be specified
            /// together with `--path`. This option refers to the `Cargo.toml` document
            /// of a crate ("manifest").
            path.parent()
                .ok_or_else(|| CargoMSRVError::Path(PathError::NoParent(path.to_path_buf())))
                .map(Path::to_path_buf)
        } else {
            /// Otherwise, fall back to the current directory.
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
    /// The path to the Cargo manifest
    pub fn manifest(&self) -> Utf8PathBuf {
        self.crate_path.join("Cargo.toml")
    }

    /// The path to the Cargo lock file
    pub fn lock(&self) -> Utf8PathBuf {
        self.crate_path.join("Cargo.lock")
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DebugOutputContext {
    /// The logging options to use, or `None` if
    /// no logging should be performed.
    pub logging: Option<Logging>,
}

impl From<DebugOutputOpts> for DebugOutputContext {
    fn from(opts: DebugOutputOpts) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct Logging {
    /// The severity of the logs
    pub log_level: LogLevel,

    /// The place where the program will put its logs
    pub log_target: TracingTargetOption,
}
