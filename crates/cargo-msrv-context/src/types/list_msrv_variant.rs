use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ListMsrvVariant {
    DirectDeps,
    #[default]
    OrderedByMSRV,
}

pub const DIRECT_DEPS: &str = "direct-deps";
pub const ORDERED_BY_MSRV: &str = "ordered-by-msrv";

impl FromStr for ListMsrvVariant {
    type Err = ParseListMsrvVariantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            DIRECT_DEPS => Self::DirectDeps,
            ORDERED_BY_MSRV => Self::OrderedByMSRV,
            elsy => {
                return Err(ParseListMsrvVariantError(elsy.to_string()));
            }
        })
    }
}

impl fmt::Display for ListMsrvVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectDeps => write!(f, "{}", DIRECT_DEPS),
            Self::OrderedByMSRV => write!(f, "{}", ORDERED_BY_MSRV),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("No such list variant '{0}'")]
pub struct ParseListMsrvVariantError(pub String);
