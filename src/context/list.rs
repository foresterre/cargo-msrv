use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::config::list::ListMsrvVariant;
use crate::context::{
    CustomCheckContext, DebugOutputContext, EnvironmentContext, RustReleasesContext,
    ToolchainContext, UserOutputContext,
};
use std::convert::TryInto;

#[derive(Debug)]
pub struct ListContext {
    /// The type of output expected by the user
    pub variant: ListMsrvVariant,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}

impl From<CargoMsrvOpts> for ListContext {
    fn from(opts: CargoMsrvOpts) -> Self {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let subcommand = match subcommand {
            Some(SubCommand::List(opts)) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `list`!"),
        };

        Self {
            variant: subcommand.variant,
            environment: (&shared_opts).try_into().unwrap(), // todo!
            user_output: shared_opts.user_output_opts.into(),
            debug_output: shared_opts.debug_output_opts.into(),
        }
    }
}
