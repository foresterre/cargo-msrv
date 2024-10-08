use crate::error::{IoError, IoErrorSource};
use crate::reporter::event::{
    AuxiliaryOutput, AuxiliaryOutputItem, Destination, ToolchainFileKind,
};
use crate::reporter::Reporter;
use crate::{semver, TResult};
use camino::{Utf8Path, Utf8PathBuf};
use std::fmt;

const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

// - consider: replace toolchain file with 'best' rust-toolchain(.toml) format variant available
// - consider: support old toolchain-file format
// - consider: do not simply override, also support components, targets, profile
//     - in reverse: use the values from rust-toolchain file to auto configure config
pub fn write_toolchain_file(
    reporter: &impl Reporter,
    stable_version: &semver::Version,
    crate_root: &Utf8Path,
) -> TResult<()> {
    let path = toolchain_file(crate_root);
    let content = format_toolchain_file(stable_version);

    std::fs::write(&path, content).map_err(|error| IoError {
        error,
        source: IoErrorSource::WriteFile(path.clone()),
    })?;

    reporter.report_event(AuxiliaryOutput::new(
        Destination::file(path),
        AuxiliaryOutputItem::toolchain_file(ToolchainFileKind::Toml),
    ))?;

    Ok(())
}

/// Determine whether we should use a .toml extension or no extension for the rust-toolchain file.
fn toolchain_file(path: &Utf8Path) -> Utf8PathBuf {
    fn without_extension(path: &Utf8Path) -> Option<Utf8PathBuf> {
        let file = path.join(TOOLCHAIN_FILE);
        file.exists().then_some(file)
    }

    fn with_extension(path: &Utf8Path) -> Option<Utf8PathBuf> {
        let file = path.join(TOOLCHAIN_FILE_TOML);
        file.exists().then_some(file)
    }

    // Without extension variant has precedence over with extension variant
    // https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file
    without_extension(path)
        .or_else(|| with_extension(path))
        .unwrap_or_else(|| path.join(TOOLCHAIN_FILE))
}

fn format_toolchain_file<D>(channel: &D) -> String
where
    D: fmt::Display,
{
    format!(
        r#"[toolchain]
channel = "{}"
"#,
        channel
    )
}

#[cfg(test)]
mod write_toolchain_file_tests {
    use super::IoError;
    use crate::error::IoErrorSource;
    use crate::reporter::event::{
        AuxiliaryOutput, AuxiliaryOutputItem, Destination, ToolchainFileKind,
    };
    use crate::reporter::{FakeTestReporter, TestReporterWrapper};
    use crate::writer::toolchain_file::write_toolchain_file;
    use crate::{semver, CargoMSRVError, Event};
    use assert_fs::prelude::*;
    use camino::{Utf8Path, Utf8PathBuf};

    #[test]
    fn no_toolchain_file_yet() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 22, 44);

        let toolchain_file = tmp.join("rust-toolchain");
        assert!(!toolchain_file.exists()); // should not exist yet

        write_toolchain_file(&fake_reporter, &version, root).unwrap();
        assert!(toolchain_file.exists()); // now should exist

        let contents = std::fs::read_to_string(&toolchain_file).unwrap();
        let expected = r#"[toolchain]
channel = "1.22.44"
"#;
        assert_eq!(&contents, expected);

        let toolchain_file_toml = tmp.join("rust-toolchain.toml");
        assert!(!toolchain_file_toml.exists()); // toml variant should not exist
    }

    #[test]
    fn pre_existing_without_extension() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain").touch().unwrap();

        let root = tmp.path();
        let root = Utf8Path::from_path(root).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 33, 55);

        let toolchain_file = tmp.join("rust-toolchain");
        let metadata = std::fs::metadata(&toolchain_file).unwrap(); // panics if file does not exist
        assert_eq!(metadata.len(), 0); // created empty

        write_toolchain_file(&fake_reporter, &version, root).unwrap();
        assert!(toolchain_file.exists()); // should still exist

        let contents = std::fs::read_to_string(&toolchain_file).unwrap();
        let expected = r#"[toolchain]
channel = "1.33.55"
"#;
        assert_eq!(&contents, expected);

        let toolchain_file_toml = tmp.join("rust-toolchain.toml");
        assert!(!toolchain_file_toml.exists()); // toml variant should not exist
    }

    #[test]
    fn pre_existing_with_extension() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain.toml").touch().unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 44, 66);

        let toolchain_file_toml = tmp.join("rust-toolchain.toml");
        let metadata = std::fs::metadata(&toolchain_file_toml).unwrap(); // panics if file does not exist
        assert_eq!(metadata.len(), 0); // created empty

        write_toolchain_file(&fake_reporter, &version, root).unwrap();
        assert!(toolchain_file_toml.exists()); // should still exist

        let contents = std::fs::read_to_string(&toolchain_file_toml).unwrap();
        let expected = r#"[toolchain]
channel = "1.44.66"
"#;
        assert_eq!(&contents, expected);

        let toolchain_file = tmp.join("rust-toolchain");
        assert!(!toolchain_file.exists()); // no ext variant should not exist
    }

    #[test]
    fn without_extension_takes_precedence() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain").touch().unwrap();
        tmp.child("rust-toolchain.toml").touch().unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 55, 77);

        let toolchain_file = tmp.join("rust-toolchain");
        let toolchain_file_toml = tmp.join("rust-toolchain.toml");

        let metadata = std::fs::metadata(&toolchain_file).unwrap(); // panics if file does not exist
        let metadata_toml = std::fs::metadata(&toolchain_file_toml).unwrap(); // panics if file does not exist
        assert_eq!(metadata.len(), 0); // created empty
        assert_eq!(metadata_toml.len(), 0); // created empty

        write_toolchain_file(&fake_reporter, &version, root).unwrap();
        assert!(toolchain_file.exists()); // should still exist
        assert!(toolchain_file_toml.exists()); // should still exist

        // check contents
        let contents = std::fs::read_to_string(&toolchain_file).unwrap();
        let expected = r#"[toolchain]
channel = "1.55.77"
"#;
        assert_eq!(&contents, expected);

        // check that toml contents are still empty
        let metadata_toml = std::fs::metadata(&toolchain_file_toml).unwrap();
        assert_eq!(metadata_toml.len(), 0);
    }

    #[test]
    fn check_reporter_event() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain").touch().unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let test_reporter = TestReporterWrapper::default();
        let version = semver::Version::new(2, 0, 5);

        write_toolchain_file(test_reporter.get(), &version, root).unwrap();

        let events = test_reporter.wait_for_events();
        let expected: Vec<Event> = vec![AuxiliaryOutput::new(
            Destination::file(Utf8PathBuf::from_path_buf(tmp.join("rust-toolchain")).unwrap()),
            AuxiliaryOutputItem::toolchain_file(ToolchainFileKind::Toml),
        )
        .into()];

        phenomenon::contains_at_least_ordered(events, expected).assert_this();
    }

    #[test]
    fn write_failure() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain").create_dir_all().unwrap(); // dir so write will fail

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(2, 0, 5);

        let error = write_toolchain_file(&fake_reporter, &version, root).unwrap_err();
        let expected_path = tmp.join("rust-toolchain");

        assert!(matches!(
            error,
            CargoMSRVError::Io(IoError {
                error: _,
                source: IoErrorSource::WriteFile(_),
            })
        ));

        let message = format!("{error}");
        let check = message.contains({
            format!(
                "caused by: 'Unable to write file '{}'",
                expected_path.display()
            )
            .as_str()
        });

        assert!(check);
    }
}

#[cfg(test)]
mod toolchain_file_tests {
    use crate::writer::toolchain_file::toolchain_file;
    use assert_fs::prelude::*;
    use camino::Utf8Path;

    #[test]
    fn without_extension() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain").touch().unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let toolchain_file = toolchain_file(root);

        assert_eq!(toolchain_file, root.join("rust-toolchain"));
    }

    #[test]
    fn with_extension() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("rust-toolchain.toml").touch().unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let toolchain_file = toolchain_file(root);

        assert_eq!(toolchain_file, root.join("rust-toolchain.toml"));
    }

    #[test]
    fn default() {
        let tmp = assert_fs::TempDir::new().unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let toolchain_file = toolchain_file(root);

        assert_eq!(toolchain_file, root.join("rust-toolchain"));
    }
}

#[cfg(test)]
mod format_toolchain_file_tests {
    use crate::writer::toolchain_file::format_toolchain_file;
    use std::fmt;

    #[yare::parameterized(
        str_value = { Box::new("1.36.0") },
        semver = { Box::new(crate::semver::Version::new(1, 36, 0)) },
        bare_version = { Box::new(crate::manifest::bare_version::BareVersion::ThreeComponents(1, 36, 0))},
    )]
    fn values_which_impl_display(channel: Box<dyn fmt::Display>) {
        let content = format_toolchain_file(&channel);
        let expected = r#"[toolchain]
channel = "1.36.0"
"#;
        assert_eq!(&content, expected);
    }
}
