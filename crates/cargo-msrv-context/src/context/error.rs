use crate::types::ParseEditionError;
use camino::Utf8PathBuf;
use cargo_msrv_manifest::ManifestParseError;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

pub type TResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("The default host triple (target) could not be found.")]
    DefaultHostTripleNotFound,

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    ManifestParseError(#[from] ManifestParseError),

    #[error("Unable to find key 'package.rust-version' (or 'package.metadata.msrv') in '{0}'")]
    NoMSRVKeyInCargoToml(Utf8PathBuf),

    #[error(transparent)]
    ParseEdition(#[from] ParseEditionError),

    #[error("Unable to parse Cargo.toml: {0}")]
    ParseToml(#[from] toml_edit::TomlError),

    #[error(transparent)]
    Path(#[from] PathError),
}

#[derive(Debug, thiserror::Error)]
#[error("IO error: '{error}'. caused by: '{source}'.")]
pub struct IoError {
    pub error: io::Error,
    pub source: IoErrorSource,
}

#[derive(Debug, thiserror::Error)]
pub enum IoErrorSource {
    #[error("Unable to determine current working directory")]
    CurrentDir,

    #[error("Unable to open file '{0}'")]
    OpenFile(Utf8PathBuf),

    #[error("Unable to read file '{0}'")]
    ReadFile(Utf8PathBuf),

    #[error("Unable to write file '{0}'")]
    WriteFile(Utf8PathBuf),

    #[error("Unable to remove file '{0}'")]
    RemoveFile(Utf8PathBuf),

    #[error("Unable to rename file '{0}'")]
    RenameFile(Utf8PathBuf),

    #[error("Unable to spawn process '{0:?}'")]
    SpawnProcess(OsString),

    #[error("Unable to collect output from '{0:?}', or process did not terminate properly")]
    WaitForProcessAndCollectOutput(OsString),
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("'{}' does not exist", .0.display())]
    DoesNotExist(PathBuf),

    #[error("No parent directory for '{}'", .0.display())]
    NoParent(PathBuf),

    #[error(transparent)]
    InvalidUtf8(#[from] InvalidUtf8Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct InvalidUtf8Error {
    error: Utf8PathErrorInner,
}

impl From<camino::FromPathError> for InvalidUtf8Error {
    fn from(value: camino::FromPathError) -> Self {
        Self {
            error: Utf8PathErrorInner::FromPath(value),
        }
    }
}

impl From<camino::FromPathBufError> for InvalidUtf8Error {
    fn from(value: camino::FromPathBufError) -> Self {
        Self {
            error: Utf8PathErrorInner::FromPathBuf(value),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Utf8PathErrorInner {
    #[error("Path contains non UTF-8 characters")]
    FromPath(camino::FromPathError),
    #[error("Path contains non UTF-8 characters (path: '{}')", .0.as_path().display())]
    FromPathBuf(camino::FromPathBufError),
}
