use std::str::FromStr;

use clap::ArgMatches;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
pub struct ListCmdConfig {
    #[builder(default)]
    pub variant: ListVariant,
}

impl ListCmdConfig {
    pub fn try_from_args(args: &ArgMatches) -> Result<Self, crate::CargoMSRVError> {
        use crate::cli::id;

        let variant = if let Some(var) = args.value_of(id::SUB_COMMAND_LIST_VARIANT) {
            let typ = ListVariant::from_str(var)?;
            Some(typ)
        } else {
            None
        };

        let config = ListCmdConfig::builder()
            .variant(variant.unwrap_or_default())
            .build();

        Ok(config)
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
                )));
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
