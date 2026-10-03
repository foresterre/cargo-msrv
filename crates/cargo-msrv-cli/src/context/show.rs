use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::make_environment_ctx;
use cargo_msrv_context::ShowContext;
use cargo_msrv_context::context::error::{Error, TResult};
use std::convert::TryFrom;

impl TryFrom<CargoMsrvOpts> for ShowContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
        } = opts;

        let show_opts = match subcommand {
            SubCommand::Show(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `show`!"),
        };

        Ok(Self {
            environment: make_environment_ctx(&shared_opts, &show_opts.workspace)?,
        })
    }
}
