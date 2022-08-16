use std::fmt::Formatter;
use std::{fmt, str::FromStr};

#[derive(Clone, Debug)]
pub struct ListCmdConfig {
    pub variant: ListMsrvVariant,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListMsrvVariant {
    DirectDeps,
    OrderedByMSRV,
}

pub(crate) const DIRECT_DEPS: &str = "direct-deps";
pub(crate) const ORDERED_BY_MSRV: &str = "ordered-by-msrv";

impl FromStr for ListMsrvVariant {
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

impl fmt::Display for ListMsrvVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectDeps => write!(f, "{}", DIRECT_DEPS),
            Self::OrderedByMSRV => write!(f, "{}", ORDERED_BY_MSRV),
        }
    }
}

impl ListMsrvVariant {
    pub(crate) const fn variants() -> &'static [&'static str] {
        &[DIRECT_DEPS, ORDERED_BY_MSRV]
    }
}

impl Default for ListMsrvVariant {
    fn default() -> Self {
        Self::OrderedByMSRV
    }
}
