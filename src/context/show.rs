use crate::cli::{shared_opts, CargoMsrvOpts, SubCommand};
use crate::context::list::ListContext;
use crate::context::{DebugOutputContext, EnvironmentContext, UserOutputContext};
use std::convert::TryInto;

#[derive(Debug)]
pub struct ShowContext {
    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}

impl From<CargoMsrvOpts> for ShowContext {
    fn from(opts: CargoMsrvOpts) -> Self {
        let CargoMsrvOpts { shared_opts, .. } = opts;

        Self {
            environment: (&shared_opts).try_into().unwrap(), // todo!
            user_output: shared_opts.user_output_opts.into(),
            debug_output: shared_opts.debug_output_opts.into(),
        }
    }
}
