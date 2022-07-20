use crate::error::IoErrorSource;
use crate::{CargoMSRVError, Config, TResult};
#[cfg(not(test))]
use once_cell::sync::OnceCell;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

// Not used during unit-testing, since the same program execution will be used
// across unit tests. As such, temporary paths used during unit testing can only be
// set once.
#[cfg(not(test))]
static INSTANCE: OnceCell<GlobalContext> = OnceCell::new();

#[cfg(test)]
static mut INSTANCE: Option<GlobalContext> = None;

/// Context which provides a way to lazily initialize values.
/// Once initialized, the initialized properties can be re-used.
///
/// [LazyContext::init]: crate::ctx::LazyContext::init
#[derive(Debug, Clone)]
pub struct GlobalContext {
    paths: CratePaths,
}

impl GlobalContext {
    pub fn from_config<'c>(config: &'c Config<'c>) -> TResult<&Self> {
        #[cfg(not(test))]
        {
            INSTANCE.get_or_try_init(|| GlobalContext::try_from(config))
        }

        // Always replace the context during tests
        #[cfg(test)]
        {
            let context = GlobalContext::try_from(config)?;

            // Not thread safe! FIXME: Not ideal, try to find a better solution, perhaps using ArcSwap?
            // SAFETY: Only used during tests, which we run using `--test-threads=1`
            unsafe {
                let _ = INSTANCE.replace(context);
                Ok(INSTANCE.as_ref().unwrap())
            }
        }
    }

    /// Get the manifest path from the crate root folder.
    ///
    /// # Panics
    ///
    /// Panics if the inner context has not been initialized
    pub fn manifest_path(&self) -> &Path {
        self.paths.manifest_path.path()
    }

    /// Get the crate root path from the crate root folder.
    ///
    /// # Panics
    ///
    /// Panics if the inner context has not been initialized
    pub fn crate_path(&self) -> &Path {
        self.paths.crate_path.path()
    }
}

impl<'c> TryFrom<&'c Config<'c>> for GlobalContext {
    type Error = CargoMSRVError;

    fn try_from(config: &'c Config<'c>) -> Result<Self, Self::Error> {
        let paths = CratePaths::try_from(config)?;

        Ok(Self { paths })
    }
}

#[derive(Debug, Clone)]
struct CratePaths {
    manifest_path: ManifestPath,
    crate_path: CratePath,
}

impl<'c> TryFrom<&'c Config<'c>> for CratePaths {
    type Error = CargoMSRVError;

    fn try_from(config: &'c Config<'c>) -> Result<Self, Self::Error> {
        let manifest_path = ManifestPath::try_from(config)?;
        let crate_path = CratePath::try_from(config)?;

        Ok(CratePaths {
            manifest_path,
            crate_path,
        })
    }
}

#[derive(Debug, Clone)]
struct ManifestPath(PathBuf);

impl ManifestPath {
    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

impl<'c> TryFrom<&'c Config<'c>> for ManifestPath {
    type Error = CargoMSRVError;

    fn try_from(config: &'c Config<'c>) -> Result<Self, Self::Error> {
        fn from_manifest_path(path: &Path) -> TResult<ManifestPath> {
            Ok(ManifestPath(path.to_path_buf()))
        }

        fn from_crate_path(path: &Path) -> TResult<ManifestPath> {
            Ok(ManifestPath(path.join("Cargo.toml")))
        }

        fn from_current_dir() -> TResult<ManifestPath> {
            std::env::current_dir()
                .map(|path| path.join("Cargo.toml"))
                .map(ManifestPath)
                .map_err(|error| CargoMSRVError::Io {
                    error,
                    source: IoErrorSource::CurrentDir,
                })
        }

        // Either the manifest_path or crate_path (or neither) should be set
        match (config._manifest_path(), config._crate_path()) {
            (Some(path), _) => from_manifest_path(path),
            (_, Some(path)) => from_crate_path(path),
            _ => from_current_dir(),
        }
    }
}

#[derive(Debug, Clone)]
struct CratePath(PathBuf);

impl CratePath {
    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

impl<'c> TryFrom<&'c Config<'c>> for CratePath {
    type Error = CargoMSRVError;

    fn try_from(config: &'c Config<'c>) -> Result<Self, Self::Error> {
        fn from_manifest_path(path: &Path) -> TResult<CratePath> {
            path.parent()
                .ok_or_else(|| CargoMSRVError::CrateRootDir(path.to_path_buf()))
                .map(Path::to_path_buf)
                .map(CratePath)
        }

        fn from_crate_path(path: &Path) -> TResult<CratePath> {
            Ok(CratePath(path.to_path_buf()))
        }

        fn from_current_dir() -> TResult<CratePath> {
            std::env::current_dir()
                .map(CratePath)
                .map_err(|error| CargoMSRVError::Io {
                    error,
                    source: IoErrorSource::CurrentDir,
                })
        }

        // Either the manifest_path or crate_path (or neither) should be set
        match (config._manifest_path(), config._crate_path()) {
            (Some(path), _) => from_manifest_path(path),
            (_, Some(path)) => from_crate_path(path),
            _ => from_current_dir(),
        }
    }
}
