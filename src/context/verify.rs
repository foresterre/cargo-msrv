use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::{
    CheckCmdContext, EnvironmentContext, RustReleasesContext, ToolchainContext, UserOutputContext,
};

use crate::error::CargoMSRVError;
use crate::sub_command::verify::RustVersion;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct VerifyContext {
    /// The resolved Rust version, to check against for toolchain compatibility.
    pub rust_version: RustVersion,

    /// Ignore the lockfile for the MSRV verification
    pub ignore_lockfile: bool,

    /// Don't print the result of compatibility check
    pub no_check_feedback: bool,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for custom checks to be used with rustup
    pub check_cmd: CheckCmdContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,
}

impl TryFrom<CargoMsrvOpts> for VerifyContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts {
            find_opts,
            shared_opts,
            subcommand,
        } = opts;

        let subcommand = match subcommand {
            Some(SubCommand::Verify(opts)) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `verify`!"),
        };

        let toolchain = find_opts.toolchain_opts.try_into()?;
        let environment = (&shared_opts).try_into()?;

        let rust_version = match subcommand.rust_version {
            Some(v) => RustVersion::from_arg(v),
            None => RustVersion::try_from_environment(&environment)?,
        };

        Ok(Self {
            rust_version,
            ignore_lockfile: find_opts.ignore_lockfile,
            no_check_feedback: find_opts.no_check_feedback,
            rust_releases: find_opts.rust_releases_opts.into(),
            toolchain,
            check_cmd: find_opts.custom_check_opts.into(),
            environment,
            user_output: shared_opts.user_output_opts.into(),
        })
    }
}
