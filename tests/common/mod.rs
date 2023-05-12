#![allow(dead_code)]

use std::path::PathBuf;

pub mod fixture;
pub mod reporter;
pub mod runner;
pub mod sub_cmd_find;
pub mod sub_cmd_verify;

pub fn fixtures_path() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}
