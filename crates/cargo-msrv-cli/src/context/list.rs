use crate::cli::{CargoMsrvOpts, SubCommand};
use cargo_msrv_context::ListContext;
use cargo_msrv_context::context::error::{Error, TResult};
use std::convert::{TryFrom, TryInto};

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

        let environment = (&shared_opts).try_into()?;

        Ok(Self {
            variant: list_opts.variant,
            environment,
        })
    }
}
