use cargo_msrv_types::BareVersion;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Edition {
    Edition2015,
    Edition2018,
    Edition2021,
    Edition2024,
}

impl FromStr for Edition {
    type Err = ParseEditionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "2015" => Ok(Self::Edition2015),
            "2018" => Ok(Self::Edition2018),
            "2021" => Ok(Self::Edition2021),
            "2024" => Ok(Self::Edition2024),
            unknown => Err(ParseEditionError::UnknownEdition(unknown.to_string())),
        }
    }
}

impl Edition {
    pub fn as_bare_version(&self) -> BareVersion {
        match self {
            Self::Edition2015 => BareVersion::ThreeComponents(1, 0, 0),
            Self::Edition2018 => BareVersion::ThreeComponents(1, 31, 0),
            Self::Edition2021 => BareVersion::ThreeComponents(1, 56, 0),
            // Actual stable version is pending; planning: https://doc.rust-lang.org/nightly/edition-guide/rust-2024/index.html
            Self::Edition2024 => BareVersion::ThreeComponents(1, 85, 0),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseEditionError {
    #[error("Edition '{0}' is not supported")]
    UnknownEdition(String),
}
