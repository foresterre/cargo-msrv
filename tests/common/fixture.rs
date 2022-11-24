use crate::common::fixtures_path;
use std::path::{Path, PathBuf};
use std::{fs, io};
use test_dir::{DirBuilder, TestDir};

// The temp dir will be cleaned up when this handle goes out of scope.
pub struct TestDirHandle(TestDir);

impl TestDirHandle {
    /// The path of the temp. test directory.
    pub fn path(&self) -> &Path {
        self.0.root()
    }

    /// Convenience method to return the `&str` representation of the path of the temp. test directory.
    ///
    /// **Panics**
    ///
    /// Panics if the inner path can not be converted to a `&str`.
    pub fn path_as_str(&self) -> &str {
        self.path()
            .to_str()
            .expect("Unable to convert TestDirHandle path to &str")
    }
}

/// Creates a new temp. test directory and copies the files and sub-directories from the given `path` to it.
/// The given `fixture_path` should be located within the `$crate/tests/fixtures` directory (which
/// itself is prepended to the given fixture path.
///
/// Otherwise does the same as [`copy_to_test_dir`]
///
/// [`copy_to_test_dir`]: crate::common::fixture::copy_to_test_dir
pub fn copy_fixture_to_test_dir<P: AsRef<Path>>(fixture_path: P) -> TestDirHandle {
    let path = fixtures_path().join(fixture_path);
    copy_to_test_dir(path)
}

/// Creates a new temp. test directory and copies the files and sub-directories from the given `path` to it.
/// The files and sub-directories will be rooted in the temp. directory from the given path.
pub fn copy_to_test_dir<P: AsRef<Path>>(path: P) -> TestDirHandle {
    fn _copy_to_test_dir(path: &Path) -> TestDirHandle {
        let test_dir = TestDir::temp();

        copy_recursive(test_dir.root(), path).expect("Unable to copy crate fixtures!");

        TestDirHandle(test_dir)
    }

    _copy_to_test_dir(path.as_ref())
}

fn copy_recursive(root: &Path, fixture_path: &Path) -> Result<(), Error> {
    for entry in fs::read_dir(fixture_path).map_err(|_| Error::ReadDir {
        path: fixture_path.to_path_buf(),
    })? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && !path.ends_with("target") {
            let to = strike_root(&path, root);
            fs::create_dir(&to)?;

            copy_recursive(&to, &path)?;
        } else if path.is_file() {
            let to = strike_root(&path, root);
            fs::copy(path, to)?;
        } else {
            // skip
        }
    }

    Ok(())
}

fn strike_root(from: &Path, base: &Path) -> PathBuf {
    // may be a file or directory
    let item = from.file_name().unwrap();

    // join  the item to the new root
    base.join(item)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    GenericIo(io::Error),

    #[error("Unable to read directory '{}'", path.display())]
    ReadDir { path: PathBuf },
}

impl From<io::Error> for Error {
    fn from(item: io::Error) -> Self {
        Self::GenericIo(item)
    }
}
