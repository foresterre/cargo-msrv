use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseSource {
    #[cfg(feature = "rust-releases-changelog-source")]
    #[default]
    RustChangelog,
    #[cfg(feature = "rust-releases-github-source")]
    GitHub,
    #[cfg(feature = "rust-releases-dist-source")]
    RustDist,
    #[cfg_attr(
        not(any(
            feature = "rust-releases-changelog-source",
            feature = "rust-releases-github-source",
            feature = "rust-releases-dist-source"
        )),
        default
    )]
    Offline,
    #[cfg(any(
        feature = "rust-releases-changelog-source",
        feature = "rust-releases-github-source",
        feature = "rust-releases-dist-source"
    ))]
    #[cfg_attr(not(feature = "rust-releases-changelog-source"), default)]
    OfflineUnlessOutdated,
}

impl FromStr for ReleaseSource {
    type Err = ParseReleaseSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl From<ReleaseSource> for &'static str {
    fn from(value: ReleaseSource) -> Self {
        match value {
            #[cfg(feature = "rust-releases-changelog-source")]
            ReleaseSource::RustChangelog => "rust-changelog",
            #[cfg(feature = "rust-releases-github-source")]
            ReleaseSource::GitHub => "github",
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => "rust-dist",
            ReleaseSource::Offline => "offline",
            #[cfg(any(
                feature = "rust-releases-changelog-source",
                feature = "rust-releases-github-source",
                feature = "rust-releases-dist-source"
            ))]
            ReleaseSource::OfflineUnlessOutdated => "offline-unless-outdated",
        }
    }
}

impl TryFrom<&str> for ReleaseSource {
    type Error = ParseReleaseSourceError;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
        match source {
            #[cfg(feature = "rust-releases-changelog-source")]
            "rust-changelog" => Ok(Self::RustChangelog),
            #[cfg(feature = "rust-releases-github-source")]
            "github" => Ok(Self::GitHub),
            #[cfg(feature = "rust-releases-dist-source")]
            "rust-dist" => Ok(Self::RustDist),
            "offline" => Ok(Self::Offline),
            #[cfg(any(
                feature = "rust-releases-changelog-source",
                feature = "rust-releases-github-source",
                feature = "rust-releases-dist-source"
            ))]
            "offline-unless-outdated" => Ok(Self::OfflineUnlessOutdated),
            s => Err(ParseReleaseSourceError(s.to_string())),
        }
    }
}

impl fmt::Display for ReleaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "rust-releases-changelog-source")]
            Self::RustChangelog => write!(f, "rust-changelog"),
            #[cfg(feature = "rust-releases-github-source")]
            Self::GitHub => write!(f, "github"),
            #[cfg(feature = "rust-releases-dist-source")]
            Self::RustDist => write!(f, "rust-dist"),
            Self::Offline => write!(f, "offline"),
            #[cfg(any(
                feature = "rust-releases-changelog-source",
                feature = "rust-releases-github-source",
                feature = "rust-releases-dist-source"
            ))]
            Self::OfflineUnlessOutdated => write!(f, "offline-unless-outdated"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unable to parse rust-releases source from '{0}'")]
pub struct ParseReleaseSourceError(pub String);
