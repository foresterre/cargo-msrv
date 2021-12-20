use clap::ArgMatches;
use std::{convert::TryFrom, str::FromStr};

#[derive(Clone, Debug)]
pub struct ListCmdConfig {
    pub variant: ListVariant,
}

impl<'a> TryFrom<&'a ArgMatches<'a>> for ListCmdConfig {
    type Error = crate::CargoMSRVError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        use crate::cli::id;

        let variant = if let Some(var) = args.value_of(id::SUB_COMMAND_LIST_VARIANT) {
            ListVariant::from_str(var)?
        } else {
            ListVariant::default()
        };

        Ok(ListCmdConfig { variant })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ListVariant {
    DirectDeps,
    OrderedByMSRV,
}

pub(crate) const DIRECT_DEPS: &str = "direct-deps";
pub(crate) const ORDERED_BY_MSRV: &str = "ordered-by-msrv";

impl FromStr for ListVariant {
    type Err = crate::CargoMSRVError;

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

impl ListVariant {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::DirectDeps => DIRECT_DEPS,
            Self::OrderedByMSRV => ORDERED_BY_MSRV,
        }
    }
}

impl Default for ListVariant {
    fn default() -> Self {
        Self::OrderedByMSRV
    }
}
