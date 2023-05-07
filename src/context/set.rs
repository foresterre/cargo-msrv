use crate::cli::{shared_opts, CargoMsrvOpts, SubCommand};
use crate::context::list::ListContext;
use crate::context::{DebugOutputContext, EnvironmentContext, UserOutputContext};
use crate::manifest::bare_version::BareVersion;
use std::convert::TryInto;

#[derive(Debug)]
pub struct SetContext {
    /// MSRV to set.
    pub msrv: BareVersion,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}

impl From<CargoMsrvOpts> for SetContext {
    fn from(opts: CargoMsrvOpts) -> Self {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let subcommand = match subcommand {
            Some(SubCommand::Set(opts)) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `set`!"),
        };

        Self {
            msrv: subcommand.msrv,
            environment: (&shared_opts).try_into().unwrap(), // todo!
            user_output: shared_opts.user_output_opts.into(),
            debug_output: shared_opts.debug_output_opts.into(),
        }
    }
}
