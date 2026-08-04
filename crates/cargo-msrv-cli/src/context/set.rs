use crate::cli::{CargoMsrvOpts, SubCommand};
use cargo_msrv_context::SetContext;
use cargo_msrv_context::context::error::{Error, TResult};
use std::convert::{TryFrom, TryInto};

impl TryFrom<CargoMsrvOpts> for SetContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let set_opts = match subcommand {
            SubCommand::Set(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `set`!"),
        };

        let environment = (&shared_opts).try_into()?;

        Ok(Self {
            msrv: set_opts.msrv,
            rust_releases: set_opts.rust_releases_opts.into(),
            environment,
        })
    }
}
