use std::path::PathBuf;

use crate::{
    config::Config,
    errors::{CargoMSRVError, IoErrorSource, TResult},
};

pub fn crate_root_folder(config: &Config) -> TResult<PathBuf> {
    if let Some(path) = config.crate_path() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir().map_err(|error| CargoMSRVError::Io {
            error,
            source: IoErrorSource::CurrentDir,
        })
    }
}
