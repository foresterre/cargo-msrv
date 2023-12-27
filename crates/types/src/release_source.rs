use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, thiserror::Error)]
pub enum Error {
    #[error("Unable to parse rust-releases source from '{0}'")]
    RustReleasesSourceParseError(String),
}

// #[derive(ValueEnum)] // TODO
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseSource {
    #[default]
    RustChangelog,
    #[cfg(feature = "rust-releases-dist-source")]
    RustDist,
}

impl FromStr for ReleaseSource {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl From<ReleaseSource> for &'static str {
    fn from(value: ReleaseSource) -> Self {
        match value {
            ReleaseSource::RustChangelog => "rust-changelog",
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => "rust-dist",
        }
    }
}

impl TryFrom<&str> for ReleaseSource {
    type Error = Error;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
        match source {
            "rust-changelog" => Ok(Self::RustChangelog),
            #[cfg(feature = "rust-releases-dist-source")]
            "rust-dist" => Ok(Self::RustDist),
            s => Err(Error::RustReleasesSourceParseError(s.to_string())),
        }
    }
}

impl fmt::Display for ReleaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustChangelog => write!(f, "rust-changelog"),
            #[cfg(feature = "rust-releases-dist-source")]
            Self::RustDist => write!(f, "rust-dist"),
        }
    }
}
