use crate::types::ReleaseSource;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BundledFallback {
    pub max_age: BundledMaxAge,
    pub source: FallbackReleaseSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BundledMaxAge {
    days: u32,
}

impl BundledMaxAge {
    const DEFAULT_DAYS: u32 = 7;

    pub const fn from_days(days: u32) -> Self {
        Self { days }
    }

    pub const fn days(self) -> u32 {
        self.days
    }
}

impl Default for BundledMaxAge {
    fn default() -> Self {
        Self::from_days(Self::DEFAULT_DAYS)
    }
}

impl FromStr for BundledMaxAge {
    type Err = ParseBundledMaxAgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>()
            .map(Self::from_days)
            .map_err(|error| ParseBundledMaxAgeError(s.to_string(), error))
    }
}

impl fmt::Display for BundledMaxAge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.days)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unable to parse '{0}' as a number of days: {1}")]
pub struct ParseBundledMaxAgeError(String, #[source] ParseIntError);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FallbackReleaseSource {
    #[cfg(feature = "rust-releases-changelog-source")]
    #[default]
    RustChangelog,
    #[cfg(feature = "rust-releases-github-source")]
    #[cfg_attr(not(feature = "rust-releases-changelog-source"), default)]
    GitHub,
    #[cfg(feature = "rust-releases-dist-source")]
    #[cfg_attr(
        not(any(
            feature = "rust-releases-changelog-source",
            feature = "rust-releases-github-source"
        )),
        default
    )]
    RustDist,
}

impl From<FallbackReleaseSource> for ReleaseSource {
    fn from(value: FallbackReleaseSource) -> Self {
        match value {
            #[cfg(feature = "rust-releases-changelog-source")]
            FallbackReleaseSource::RustChangelog => ReleaseSource::RustChangelog,
            #[cfg(feature = "rust-releases-github-source")]
            FallbackReleaseSource::GitHub => ReleaseSource::GitHub,
            #[cfg(feature = "rust-releases-dist-source")]
            FallbackReleaseSource::RustDist => ReleaseSource::RustDist,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rust-releases-changelog-source")]
    #[test]
    fn default_fallback_is_the_default_release_source() {
        assert_eq!(
            ReleaseSource::from(FallbackReleaseSource::default()),
            ReleaseSource::default()
        );
    }

    #[yare::parameterized(
        zero = { "0", 0 },
        six_weeks = { "42", 42 },
        max = { "4294967295", u32::MAX },
    )]
    fn parse_max_age(input: &str, expected_days: u32) {
        let max_age = input.parse::<BundledMaxAge>().unwrap();

        assert_eq!(max_age.days(), expected_days);
    }

    #[yare::parameterized(
        empty = { "" },
        negative = { "-1" },
        fraction = { "1.5" },
        with_unit = { "42d" },
        too_large = { "4294967296" },
    )]
    fn parse_invalid_max_age(input: &str) {
        assert!(input.parse::<BundledMaxAge>().is_err());
    }

    #[test]
    fn default_max_age_is_two_weeks() {
        assert_eq!(BundledMaxAge::default().days(), 14);
    }
}
