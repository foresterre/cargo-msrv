use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::{
    CheckCommandContext, EnvironmentContext, RustReleasesContext, ToolchainContext,
};

use crate::check::RunCommand;
use crate::error::CargoMSRVError;
use crate::external_command::cargo_command::CargoCommand;
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
    pub check_cmd: CheckCommandContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
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
        })
    }
}

impl VerifyContext {
    pub fn run_command(&self) -> RunCommand {
        if let Some(custom) = &self.check_cmd.rustup_command {
            RunCommand::custom(custom.clone())
        } else {
            let cargo_command = CargoCommand::default()
                .target(Some(self.toolchain.target))
                .features(self.check_cmd.cargo_features.clone())
                .all_features(self.check_cmd.cargo_all_features)
                .no_default_features(self.check_cmd.cargo_no_default_features);

            RunCommand::default(cargo_command)
        }
    }
}

#[cfg(test)]
mod tests {
    mod issue_936_target {
        use crate::cli::CargoCli;
        use crate::context::VerifyContext;
        use std::convert::TryFrom;

        #[test]
        fn target_at_top_level() {
            let opts = CargoCli::parse_args(["cargo", "msrv", "--target", "x", "verify"]);
            let context = VerifyContext::try_from(opts.to_cargo_msrv_cli().to_opts()).unwrap();

            assert_eq!(context.toolchain.target, "x");
        }

        #[test]
        fn target_at_subcommand_level() {
            let opts = CargoCli::parse_args(["cargo", "msrv", "verify", "--target", "x"]);
            let context = VerifyContext::try_from(opts.to_cargo_msrv_cli().to_opts()).unwrap();

            assert_eq!(context.toolchain.target, "x");
        }

        // The subcommand takes precedence over the top-level (I would have preferred for Clap to reject the presence on both, since the num args is 0..1 with Option<T>)
        #[test]
        fn target_at_top_level_and_subcommand_level() {
            let opts =
                CargoCli::parse_args(["cargo", "msrv", "--target", "x", "verify", "--target", "y"]);
            let context = VerifyContext::try_from(opts.to_cargo_msrv_cli().to_opts()).unwrap();

            assert_eq!(context.toolchain.target, "y");
        }
    }
}
