use crate::manifest::bare_version;
use crate::manifest::bare_version::BareVersion;
use crate::ReleaseSource;
use clap::Args;
use std::str::FromStr;

#[derive(Debug, Args)]
#[command(next_help_heading = "Rust releases options")]
pub struct RustReleasesOpts {
    /// Least recent version or edition to take into account
    ///
    /// Given version must match a valid Rust toolchain, and be semver compatible,
    /// be a two component `major.minor` version. or match a Rust edition alias.
    ///
    /// For example, the edition alias "2018" would match Rust version `1.31.0`, since that's the
    /// first version which added support for the Rust 2018 edition.
    #[arg(long, value_name = "VERSION_SPEC or EDITION", alias = "minimum")]
    pub min: Option<EditionOrVersion>,

    /// Most recent version to take into account
    ///
    /// Given version must match a valid Rust toolchain, and be semver compatible, or
    /// be a two component `major.minor` version.
    #[arg(long, value_name = "VERSION_SPEC", alias = "maximum")]
    pub max: Option<BareVersion>,

    /// Include all patch releases, instead of only the last
    #[arg(long)]
    pub include_all_patch_releases: bool,

    #[arg(long, value_enum, default_value_t, value_name = "SOURCE")]
    pub release_source: ReleaseSource,
}

#[derive(Clone, Debug)]
pub enum EditionOrVersion {
    Edition(Edition),
    Version(BareVersion),
}

impl EditionOrVersion {
    pub fn as_bare_version(&self) -> BareVersion {
        match self {
            Self::Edition(edition) => edition.as_bare_version(),
            Self::Version(version) => version.clone(),
        }
    }
}

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

impl FromStr for EditionOrVersion {
    type Err = ParseEditionOrVersionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input
            .parse::<Edition>()
            .map(EditionOrVersion::Edition)
            .or_else(|edition_err| {
                BareVersion::from_str(input)
                    .map(EditionOrVersion::Version)
                    .map_err(|parse_version_err| {
                        ParseEditionOrVersionError::EditionOrVersion(
                            input.to_string(),
                            edition_err,
                            parse_version_err,
                        )
                    })
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseEditionOrVersionError {
    #[error("Value '{0}' could not be parsed as a valid Rust version: {1} + {2}")]
    EditionOrVersion(String, ParseEditionError, bare_version::Error),
}
