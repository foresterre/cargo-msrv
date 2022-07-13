use crate::error::IoErrorSource;
use crate::{CargoMSRVError, Config, TResult};
use once_cell::unsync::OnceCell;
use std::path::{Path, PathBuf};

/// Context which provides a way to lazily initialize values.
/// Once initialized, the initialized properties can be re-used.
///
/// **NB:** Must be initialized before use. See [LazyContext::init].
///
/// [LazyContext::init]: crate::ctx::LazyContext::init
#[derive(Debug, Default, Clone)]
pub struct LazyContext {
    ctx: Option<GlobalContext>,
}

impl LazyContext {
    pub fn init(&mut self, values: ContextValues) {
        self.ctx = Some(GlobalContext {
            values,
            crate_root_path: OnceCell::default(),
            manifest_path: OnceCell::default(),
        });
    }
}

impl LazyContext {
    /// Get the crate root path from the crate root folder.
    ///
    /// # Panics
    ///
    /// Panics if the inner context has not been initialized
    pub fn crate_root_path(&self) -> TResult<&Path> {
        debug_assert!(
            self.ctx.is_some(),
            "Please initialize the LazyContext before use (see LazyContext::init)"
        );

        let path = self.ctx.as_ref().unwrap().crate_root_path()?;

        Ok(path)
    }

    /// Get the manifest path from the crate root folder.
    ///
    /// # Panics
    ///
    /// Panics if the inner context has not been initialized
    pub fn manifest_path(&self) -> TResult<&Path> {
        debug_assert!(
            self.ctx.is_some(),
            "Please initialize the LazyContext before use (see LazyContext::init)"
        );

        let path = self.ctx.as_ref().unwrap().manifest_path()?;

        Ok(path)
    }
}

#[derive(Debug, Clone)]
pub struct ContextValues {
    path: Option<PathBuf>,
}

impl ContextValues {
    pub fn from_config(config: &Config) -> Self {
        Self {
            // Cloning the crate path here saves us a lot of trouble.
            // However, the real problem is that the Config is supplied through the whole program,
            // and that the context is held by the config, while it also requires values from the
            // config to determine its path.
            // Here we choose to be pragmatic, but hopefully one day we'll get to refactoring the
            // Config and LazyContext.
            path: config.crate_path().map(|it| it.to_path_buf()),
        }
    }
}

#[derive(Debug, Clone)]
struct GlobalContext {
    values: ContextValues,
    crate_root_path: OnceCell<PathBuf>,
    manifest_path: OnceCell<PathBuf>,
}

impl GlobalContext {
    pub fn crate_root_path(&self) -> TResult<&Path> {
        fn crate_root(path: Option<&Path>) -> TResult<PathBuf> {
            path.map(|p| Ok(p.to_path_buf())).unwrap_or_else(|| {
                std::env::current_dir().map_err(|error| CargoMSRVError::Io {
                    error,
                    source: IoErrorSource::CurrentDir,
                })
            })
        }

        let path = self
            .crate_root_path
            .get_or_try_init(|| crate_root(self.values.path.as_deref()))?;

        Ok(path)
    }

    pub fn manifest_path(&self) -> TResult<&Path> {
        let path = self
            .manifest_path
            .get_or_try_init(|| self.crate_root_path().map(|path| path.join("Cargo.toml")))?;

        Ok(path)
    }
}
