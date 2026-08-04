use crate::cli::{CargoMsrvOpts, SubCommand};
use cargo_msrv_context::VerifyContext;
use cargo_msrv_context::context::error::{Error, TResult};
use cargo_msrv_context::context::verify::RustVersion;
use std::convert::{TryFrom, TryInto};

impl TryFrom<CargoMsrvOpts> for VerifyContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
        } = opts;

        let verify_opts = match subcommand {
            SubCommand::Verify(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `verify`!"),
        };

        let toolchain = verify_opts.toolchain_opts.try_into()?;
        let environment = (&shared_opts).try_into()?;

        let rust_version = match verify_opts.rust_version {
            Some(v) => RustVersion::from_arg(v),
            None => RustVersion::try_from_environment(&environment)?,
        };

        Ok(Self {
            rust_version,
            ignore_lockfile: verify_opts.ignore_lockfile,
            no_check_feedback: verify_opts.no_check_feedback,
            rust_releases: verify_opts.rust_releases_opts.into(),
            toolchain,
            check_cmd: verify_opts.custom_check_opts.into(),
            environment,
        })
    }
}

#[cfg(test)]
mod tests {
    mod issue_936_target {
        use crate::cli::CargoCli;
        use cargo_msrv_context::VerifyContext;
        use std::convert::TryFrom;

        #[test]
        fn target_at_subcommand_level() {
            let opts = CargoCli::parse_args(["cargo", "msrv", "verify", "--target", "x"]);
            let context = VerifyContext::try_from(opts.to_cargo_msrv_cli().to_opts()).unwrap();

            assert_eq!(context.toolchain.target, "x");
        }
    }
}
