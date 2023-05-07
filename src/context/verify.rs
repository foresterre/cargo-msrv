use crate::cli::{shared_opts, CargoMsrvOpts, SubCommand};
use crate::context::list::ListContext;
use crate::context::{
    CustomCheckContext, DebugOutputContext, EnvironmentContext, RustReleasesContext,
    ToolchainContext, UserOutputContext,
};
use crate::manifest::bare_version::BareVersion;
use crate::sub_command::verify::RustVersion;
use clap::builder::TypedValueParser;
use std::convert::TryInto;

#[derive(Debug)]
pub struct VerifyContext {
    /// The resolved Rust version, to check against for toolchain compatibility.
    pub rust_version: BareVersion,

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

impl From<CargoMsrvOpts> for VerifyContext {
    fn from(opts: CargoMsrvOpts) -> Self {
        let CargoMsrvOpts {
            find_opts,
            shared_opts,
            subcommand,
        } = opts;

        let subcommand = match subcommand {
            Some(SubCommand::Verify(opts)) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `verify`!"),
        };

        let environment = (&shared_opts).try_into().unwrap(); // todo!

        let rust_version = match subcommand.rust_version {
            Some(v) => v,
            None => RustVersion::try_from_environment(&environment)
                .unwrap() /* todo! */
                .into_version(),
        };

        Self {
            rust_version,
            rust_releases: find_opts.rust_releases_opts.into(),
            toolchain: find_opts.toolchain_opts.into(),
            custom_check: find_opts.custom_check_opts.into(),
            environment,
            user_output: shared_opts.user_output_opts.into(),
            debug_output: shared_opts.debug_output_opts.into(),
        }
    }
}
