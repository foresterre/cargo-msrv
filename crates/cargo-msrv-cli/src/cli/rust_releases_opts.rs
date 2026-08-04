use crate::values::release_source;
use cargo_msrv_context::types::{Edition, ParseEditionError, ReleaseSource};
use cargo_msrv_types::BareVersion;
use cargo_msrv_types::bare_version;
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

    #[arg(
        long,
        value_parser = release_source::VALUES.parser(),
        default_value = release_source::VALUES.default_value(),
        value_name = "SOURCE"
    )]
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
