use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::EnvironmentContext;
use crate::error::CargoMSRVError;
use std::convert::{TryFrom, TryInto};

pub use cargo_msrv_cli::types::ListMsrvVariant;
pub(crate) use cargo_msrv_cli::types::{DIRECT_DEPS, ORDERED_BY_MSRV};

#[derive(Debug)]
pub struct ListContext {
    /// The type of output expected by the user
    pub variant: ListMsrvVariant,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}

impl TryFrom<CargoMsrvOpts> for ListContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
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
