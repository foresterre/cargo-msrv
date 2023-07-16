use crate::error::{IoError, IoErrorSource};
use crate::manifest::{CargoManifestParser, TomlParser};
use camino::Utf8Path;
use thiserror::Error;
use toml_edit::{Document, TomlError};

pub trait DocumentReader {
    fn read_document(path: impl AsRef<Utf8Path>) -> Result<Document, ManifestReaderError>;
}

#[derive(Debug)]
pub struct TomlDocumentReader;

impl DocumentReader for TomlDocumentReader {
    /// Parse the cargo manifest from the given path.
    ///
    // FIXME: We should eventually not leak the third party document type, and instead provide our own
    //  high level TOML crate agnostic API over it.
    fn read_document(path: impl AsRef<Utf8Path>) -> Result<Document, ManifestReaderError> {
        fn read_document(path: &Utf8Path) -> Result<Document, ManifestReaderError> {
            let contents = std::fs::read_to_string(path).map_err(|error| IoError {
                error,
                source: IoErrorSource::ReadFile(path.to_path_buf()),
            })?;

            CargoManifestParser
                .parse(&contents)
                .map_err(ManifestReaderError::Toml)
        }

        read_document(path.as_ref())
    }
}

#[derive(Debug, Error)]
pub enum ManifestReaderError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Toml(#[from] TomlError),
}
