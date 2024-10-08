#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub mod reporter;
pub mod runner;
pub mod sub_cmd_find;
pub mod sub_cmd_verify;

pub struct Fixture {
    fixture_dir: PathBuf,
    temp_dir: assert_fs::TempDir,
}

impl Fixture {
    pub fn new(fixture_dir: impl AsRef<Path>) -> Self {
        use assert_fs::fixture::PathCopy;

        let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(fixture_dir);

        let temp_dir = assert_fs::TempDir::new().unwrap();
        temp_dir.copy_from(&fixture_dir, &["**/*"]).unwrap();

        Self {
            fixture_dir,
            temp_dir,
        }
    }

    pub fn tmp_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.temp_dir.join(path)
    }

    pub fn to_str(&self) -> &str {
        self.temp_dir.to_str().unwrap()
    }
}
