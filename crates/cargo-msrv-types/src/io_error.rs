use camino::Utf8PathBuf;
use std::ffi::OsString;
use std::io;

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
