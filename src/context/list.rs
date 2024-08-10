use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::EnvironmentContext;
use crate::error::CargoMSRVError;
use clap::ValueEnum;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ListMsrvVariant {
    DirectDeps,
    #[default]
    OrderedByMSRV,
}

pub(crate) const DIRECT_DEPS: &str = "direct-deps";
pub(crate) const ORDERED_BY_MSRV: &str = "ordered-by-msrv";

impl FromStr for ListMsrvVariant {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            DIRECT_DEPS => Self::DirectDeps,
            ORDERED_BY_MSRV => Self::OrderedByMSRV,
            elsy => {
                return Err(crate::CargoMSRVError::InvalidConfig(format!(
                    "No such list variant '{}'",
                    elsy
                )))
            }
        })
    }
}

impl fmt::Display for ListMsrvVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectDeps => write!(f, "{}", DIRECT_DEPS),
            Self::OrderedByMSRV => write!(f, "{}", ORDERED_BY_MSRV),
        }
    }
}
