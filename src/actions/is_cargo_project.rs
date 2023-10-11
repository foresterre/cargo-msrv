use camino::Utf8Path;
use std::fs;

/// Check whether a given path can be interpreted as a Cargo project.
#[derive(Debug, Default)]
pub struct CargoProjectChecker;

impl CargoProjectChecker {
    fn is_cargo_project(&self, manifest_path: &Utf8Path) -> bool {
        fs::metadata(manifest_path.as_std_path())
            .map(|handle| handle.is_file())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_dir::{DirBuilder, FileType, TestDir};

    #[test]
    fn is_cargo_project() {
        let temp = TestDir::temp().create("Cargo.toml", FileType::EmptyFile);

        let path = temp.path("Cargo.toml");

        let is_cargo_project =
            CargoProjectChecker::default().is_cargo_project(Utf8Path::from_path(&path).unwrap());

        assert!(is_cargo_project);
    }

    #[test]
    fn is_not_cargo_project() {
        let temp = TestDir::temp();

        let path = temp.path("Cargo.toml");

        let is_cargo_project =
            CargoProjectChecker::default().is_cargo_project(Utf8Path::from_path(&path).unwrap());

        assert!(!is_cargo_project);
    }
}
