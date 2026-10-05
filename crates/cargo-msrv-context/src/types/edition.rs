use std::str::FromStr;
use version_number::Version;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
    pub fn as_version(&self) -> Version {
        match self {
            Self::Edition2015 => Version::new_full_version(1, 0, 0),
            Self::Edition2018 => Version::new_full_version(1, 31, 0),
            Self::Edition2021 => Version::new_full_version(1, 56, 0),
            // Actual stable version is pending; planning: https://doc.rust-lang.org/nightly/edition-guide/rust-2024/index.html
            Self::Edition2024 => Version::new_full_version(1, 85, 0),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseEditionError {
    #[error("Edition '{0}' is not supported")]
    UnknownEdition(String),
}
