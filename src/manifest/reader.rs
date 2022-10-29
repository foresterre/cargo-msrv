use std::path::Path;
use toml_edit::{Document, TomlError};
use crate::error::{CargoMSRVError, IoError, IoErrorSource, TResult};
use crate::manifest::{bare_version, CargoManifest, CargoManifestParser, ManifestParseError, TomlParser};
use std::convert::TryFrom;
use thiserror::Error;

pub trait ManifestReader {
    fn parse_manifest(path: &Path) -> Result<CargoManifest, ManifestReaderError>;
}

pub struct CargoManifestReader;

impl ManifestReader for CargoManifestReader {
    /// Parse the cargo manifest from the given path.
    fn parse_manifest(path: &Path) -> Result<CargoManifest, ManifestReaderError> {
        let contents = std::fs::read_to_string(path).map_err(|error| IoError {
            error,
            source: IoErrorSource::ReadFile(path.to_path_buf()),
        })?;

        let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
        CargoManifest::try_from(manifest).map_err(ManifestReaderError::ManifestParse)
    }
}

#[derive(Debug,Error)]
pub enum ManifestReaderError{

    #[error(transparent)]
    Io(#[from] IoError),


    #[error(transparent)]
    Toml(#[from] TomlError),

    #[error(transparent)]
    ManifestParse(#[from] ManifestParseError),
}
