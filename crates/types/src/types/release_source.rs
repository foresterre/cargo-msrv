use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseSource {
    RustChangelog,
    #[cfg(feature = "rust-releases-dist-source")]
    RustDist,
}

impl Default for ReleaseSource {
    fn default() -> Self {
        Self::RustChangelog
    }
}

impl ReleaseSource {
    pub(crate) fn variants() -> &'static [&'static str] {
        &[
            "rust-changelog",
            #[cfg(feature = "rust-releases-dist-source")]
            "rust-dist",
        ]
    }
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
            s => Err(Error::new(s)),
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

#[derive(Debug, thiserror::Error)]
#[error("No such release source '{}', available: '{}'", &.release_source, ReleaseSource::variants().join(","))]
pub struct Error {
    release_source: String,
}

impl Error {
    fn new(release_source: impl Into<String>) -> Self {
        Self {
            release_source: release_source.into(),
        }
    }
}
