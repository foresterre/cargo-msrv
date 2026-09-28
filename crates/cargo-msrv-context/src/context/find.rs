use crate::context::{
    CheckCommandContext, EnvironmentContext, RustReleasesContext, SearchMethod, ToolchainContext,
};
use camino::Utf8PathBuf;
use std::convert::Infallible;
use std::str::FromStr;

/// Where the heuristic search method gets the Rust versions of the standard library items from.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum HeuristicSource {
    /// The collection bundled with cargo-msrv
    #[default]
    Bundled,
    /// The latest collection, which is downloaded, and cached for a day
    Latest,
    /// A collection in a local file
    File(Utf8PathBuf),
}

impl FromStr for HeuristicSource {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bundled" => Self::Bundled,
            "latest" => Self::Latest,
            path => Self::File(Utf8PathBuf::from(path)),
        })
    }
}

#[derive(Debug)]
pub struct FindContext {
    /// Use a binary (bisect) or linear search to find the MSRV
    pub search_method: SearchMethod,

    /// Where the heuristic search method gets the Rust versions of the standard library items from
    pub heuristic_source: HeuristicSource,

    /// Write the toolchain file if the MSRV is found
    pub write_toolchain_file: bool,

    /// Ignore the lockfile for the MSRV search
    pub ignore_lockfile: bool,

    /// Don't print the result of compatibility checks
    pub no_check_feedback: bool,

    /// Treats a Rust version as incompatible when a toolchain failed to install or was otherwise unavailable
    pub skip_unavailable_toolchains: bool,

    /// Write the MSRV to the Cargo manifest
    pub write_msrv: bool,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for checks to be used with rustup
    pub check_cmd: CheckCommandContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}
