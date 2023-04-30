use crate::config::SearchMethod;
use crate::context::{
    CustomCheckContext, DebugOutputContext, EnvironmentContext, RustReleasesContext,
    ToolchainContext, UserOutputContext,
};

pub struct FindContext {
    /// Use a binary (bisect) or linear search to find the MSRV
    pub search_method: SearchMethod,

    /// Write the toolchain file if the MSRV is found
    pub write_toolchain_file: bool,

    /// Ignore the lockfile for the MSRV search
    pub ignore_lockfile: bool,

    /// Don't read the minimum edition
    pub no_read_min_edition: bool,

    /// Don't print the result of compatibility checks
    pub no_check_feedback: bool,

    /// Write the MSRV to the Cargo manifest
    pub write_msrv: bool,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for custom checks to be used with rustup
    pub custom_check: CustomCheckContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}

impl From<CargoMsrvOpts> for FindContext {
    fn from(value: CargoMsrvOpts) -> Self {
        todo!()
    }
}
