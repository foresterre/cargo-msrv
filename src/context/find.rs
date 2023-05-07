use crate::cli::{shared_opts, CargoCli, CargoMsrvOpts};
use crate::config::SearchMethod;
use crate::context::{
    CustomCheckContext, DebugOutputContext, EnvironmentContext, RustReleasesContext,
    ToolchainContext, UserOutputContext,
};
use std::convert::TryInto;

#[derive(Debug)]
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
    fn from(opts: CargoMsrvOpts) -> Self {
        let CargoMsrvOpts {
            find_opts,
            shared_opts,
            ..
        } = opts;

        Self {
            search_method: if find_opts.linear {
                SearchMethod::Linear
            } else {
                SearchMethod::Bisect
            },
            write_toolchain_file: find_opts.write_toolchain_file,
            ignore_lockfile: find_opts.ignore_lockfile,
            no_read_min_edition: find_opts.no_read_min_edition,
            no_check_feedback: find_opts.no_check_feedback,
            write_msrv: find_opts.write_msrv,
            rust_releases: find_opts.rust_releases_opts.into(),
            toolchain: find_opts.toolchain_opts.into(),
            custom_check: find_opts.custom_check_opts.into(),
            environment: (&shared_opts).try_into().unwrap(), // todo!
            user_output: shared_opts.user_output_opts.into(),
            debug_output: shared_opts.debug_output_opts.into(),
        }
    }
}
