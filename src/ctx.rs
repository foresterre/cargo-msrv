use crate::error::IoErrorSource;
use crate::{CargoMSRVError, Config, TResult};
use once_cell::unsync::OnceCell;
use std::ffi::OsStr;
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
    path: GivenPath,
}

impl ContextValues {
    pub fn from_config(config: &Config) -> Self {
        let path = if let Some(p) = config.crate_path() {
            GivenPath::new_crate_root(p)
        } else if let Some(p) = config.manifest_path() {
            GivenPath::new_manifest(p)
        } else {
            GivenPath::new_none()
        };

        Self {
            // Cloning the crate path here saves us a lot of trouble.
            // However, the real problem is that the Config is supplied through the whole program,
            // and that the context is held by the config, while it also requires values from the
            // config to determine its path.
            // Here we choose to be pragmatic, but hopefully one day we'll get to refactoring the
            // Config and LazyContext.
            path,
        }
    }
}

#[derive(Debug, Clone)]
enum GivenPath {
    CrateRoot(PathBuf),
    Manifest(PathBuf),
    None,
}

impl GivenPath {
    pub fn new_crate_root<P: AsRef<Path>>(path: P) -> Self {
        Self::CrateRoot(path.as_ref().to_path_buf())
    }

    pub fn new_manifest<P: AsRef<Path>>(path: P) -> Self {
        Self::Manifest(path.as_ref().to_path_buf())
    }

    pub fn new_none() -> Self {
        Self::None
    }

    pub fn as_crate_root(&self) -> TResult<PathBuf> {
        match self {
            Self::CrateRoot(p) => Ok(p.clone()),
            // When in a crate root, an argument "Cargo.toml" will fail, because it doesn't have a
            // parent, when the path is relative.
            Self::Manifest(p) if p.as_os_str() == OsStr::new("Cargo.toml") => {
                Ok(Path::new(".").to_path_buf())
            }
            Self::Manifest(p) => Ok(p.parent().unwrap().to_path_buf()),
            Self::None => std::env::current_dir().map_err(|error| CargoMSRVError::Io {
                error,
                source: IoErrorSource::CurrentDir,
            }),
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
        let path = self
            .crate_root_path
            .get_or_try_init(|| self.values.path.as_crate_root())?;

        Ok(path)
    }

    pub fn manifest_path(&self) -> TResult<&Path> {
        let path = self
            .manifest_path
            .get_or_try_init(|| self.crate_root_path().map(|path| path.join("Cargo.toml")))?;

        Ok(path)
    }
}
