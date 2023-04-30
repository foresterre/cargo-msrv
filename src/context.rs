//! The `context` is the resolved configuration for the current run of `cargo-msrv`.
//!
//! The context is the synthesized user input (opts).
//! Where the user input deals with presentation, the context consists of only
//! the relevant data which is necessary for the functioning of the subcommand.
//!
//! Unlike the opts, the context is top down, not bottom up.

use crate::config::{OutputFormat, ReleaseSource, TracingTargetOption};
use crate::context::find::FindContext;
use crate::context::list::ListContext;
use crate::context::set::SetContext;
use crate::context::show::ShowContext;
use crate::context::verify::VerifyContext;
use crate::log_level::LogLevel;
use crate::manifest::bare_version::BareVersion;
use camino::Utf8PathBuf;
use cargo_metadata::camino::Utf8PathBuf;

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

pub struct ToolchainContext {
    /// The target of the toolchain
    pub target: Option<String>,
}

pub struct CustomCheckContext {
    /// The custom `Rustup` command to invoke for a toolchain.
    pub custom_rustup_command: Vec<String>,
}

pub struct EnvironmentContext {
    /// The path to the root of a crate.
    ///
    /// Does not include a manifest file like Cargo.toml, so it's easy to append
    /// a file path like `Cargo.toml` or `Cargo.lock`.
    pub crate_path: Utf8PathBuf,
}

pub struct UserOutputContext {
    /// The output format to use, or `None` if
    /// no user output should be presented to the user.
    pub output_format: Option<OutputFormat>,
}

pub struct DebugOutputContext {
    /// The logging options to use, or `None` if
    /// no logging should be performed.
    pub logging: Option<Logging>,
}

struct Logging {
    /// The severity of the logs
    pub log_level: LogLevel,

    /// The place where the program will put its logs
    pub log_target: TracingTargetOption,
}
