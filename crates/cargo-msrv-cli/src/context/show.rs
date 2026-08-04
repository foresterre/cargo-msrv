use crate::cli::CargoMsrvOpts;
use cargo_msrv_context::ShowContext;
use cargo_msrv_context::context::error::{Error, TResult};
use std::convert::{TryFrom, TryInto};

impl TryFrom<CargoMsrvOpts> for ShowContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts { shared_opts, .. } = opts;

        Ok(Self {
            environment: (&shared_opts).try_into()?,
        })
    }
}
