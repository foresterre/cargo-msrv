use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::make_environment_ctx;
use cargo_msrv_context::ListContext;
use cargo_msrv_context::context::error::{Error, TResult};
use std::convert::TryFrom;

impl TryFrom<CargoMsrvOpts> for ListContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let list_opts = match subcommand {
            SubCommand::List(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `list`!"),
        };

        let environment = make_environment_ctx(&shared_opts, &clap_cargo::Workspace::default())?;

        Ok(Self {
            variant: list_opts.variant,
            environment,
        })
    }
}
