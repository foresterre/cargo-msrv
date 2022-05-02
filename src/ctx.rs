use crate::paths::crate_root_folder;
use crate::{Config, TResult};
use once_cell::unsync::OnceCell;
use std::path::{Path, PathBuf};

/// Context which provides a way to lazily initialize values.
/// Once initialized, the initialized properties can be re-used.
#[derive(Debug, Default, Clone)]
pub struct ComputedCtx {
    manifest_path: OnceCell<PathBuf>,
}

impl ComputedCtx {
    /// Get the manifest path from the crate root folder.
    pub fn manifest_path(&self, config: &Config) -> TResult<&Path> {
        let path = self
            .manifest_path
            .get_or_try_init(|| crate_root_folder(config).map(|p| p.join("Cargo.toml")))?;

        Ok(path)
    }
}
