use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::config::list::ListMsrvVariant;
use crate::context::{EnvironmentContext, UserOutputContext};
use crate::error::CargoMSRVError;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct ListContext {
    /// The type of output expected by the user
    pub variant: ListMsrvVariant,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,
}

impl TryFrom<CargoMsrvOpts> for ListContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let subcommand = match subcommand {
            Some(SubCommand::List(opts)) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `list`!"),
        };

        let environment = (&shared_opts).try_into()?;

        Ok(Self {
            variant: subcommand.variant,
            environment, // todo!
            user_output: shared_opts.user_output_opts.into(),
        })
    }
}
