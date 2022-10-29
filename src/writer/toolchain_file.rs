use crate::combinators::ThenSome;
use crate::error::{IoError, IoErrorSource};
use crate::reporter::event::{
    AuxiliaryOutput, AuxiliaryOutputItem, Destination, ToolchainFileKind,
};
use crate::reporter::Reporter;
use crate::{semver, CargoMSRVError, Config, TResult};
use std::fmt;
use std::path::{Path, PathBuf};


const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

// - consider: replace toolchain file with 'best' rust-toolchain(.toml) format variant available
// - consider: support old toolchain-file format
// - consider: do not simply override, also support components, targets, profile
//     - in reverse: use the values from rust-toolchain file to auto configure config
pub fn write_toolchain_file(
    config: &Config,
    reporter: &impl Reporter,
    stable_version: &semver::Version,
) -> TResult<()> {
    let path_prefix = config.context().crate_root_path()?;
    let path = toolchain_file(path_prefix);
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
fn toolchain_file(path: &Path) -> PathBuf {
    fn without_extension(path: &Path) -> Option<PathBuf> {
        let file = path.join(TOOLCHAIN_FILE);
        ThenSome::then_some(file.exists(), file)
    }

    fn with_extension(path: &Path) -> Option<PathBuf> {
        let file = path.join(TOOLCHAIN_FILE_TOML);
        ThenSome::then_some(file.exists(), file)
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
    use crate::config::ConfigBuilder;
    use crate::error::IoErrorSource;
    use crate::reporter::event::{
        AuxiliaryOutput, AuxiliaryOutputItem, Destination, ToolchainFileKind,
    };
    use crate::reporter::{FakeTestReporter, TestReporterWrapper};
    use crate::writer::toolchain_file::write_toolchain_file;
    use crate::{semver, CargoMSRVError, Event, SubcommandId};
    use test_dir::{DirBuilder, FileType, TestDir};
    use super::IoError;

    #[test]
    fn no_toolchain_file_yet() {
        let tmp = TestDir::temp();
        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 22, 44);

        let toolchain_file = tmp.path("rust-toolchain");
        assert!(!toolchain_file.exists()); // should not exist yet

        write_toolchain_file(&config, &fake_reporter, &version).unwrap();
        assert!(toolchain_file.exists()); // now should exist

        let contents = std::fs::read_to_string(&toolchain_file).unwrap();
        let expected = r#"[toolchain]
channel = "1.22.44"
"#;
        assert_eq!(&contents, expected);

        let toolchain_file_toml = tmp.path("rust-toolchain.toml");
        assert!(!toolchain_file_toml.exists()); // toml variant should not exist
    }

    #[test]
    fn pre_existing_without_extension() {
        let tmp = TestDir::temp().create("rust-toolchain", FileType::EmptyFile);

        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 33, 55);

        let toolchain_file = tmp.path("rust-toolchain");
        let metadata = std::fs::metadata(&toolchain_file).unwrap(); // panics if file does not exist
        assert_eq!(metadata.len(), 0); // created empty

        write_toolchain_file(&config, &fake_reporter, &version).unwrap();
        assert!(toolchain_file.exists()); // should still exist

        let contents = std::fs::read_to_string(&toolchain_file).unwrap();
        let expected = r#"[toolchain]
channel = "1.33.55"
"#;
        assert_eq!(&contents, expected);

        let toolchain_file_toml = tmp.path("rust-toolchain.toml");
        assert!(!toolchain_file_toml.exists()); // toml variant should not exist
    }

    #[test]
    fn pre_existing_with_extension() {
        let tmp = TestDir::temp().create("rust-toolchain.toml", FileType::EmptyFile);

        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 44, 66);

        let toolchain_file_toml = tmp.path("rust-toolchain.toml");
        let metadata = std::fs::metadata(&toolchain_file_toml).unwrap(); // panics if file does not exist
        assert_eq!(metadata.len(), 0); // created empty

        write_toolchain_file(&config, &fake_reporter, &version).unwrap();
        assert!(toolchain_file_toml.exists()); // should still exist

        let contents = std::fs::read_to_string(&toolchain_file_toml).unwrap();
        let expected = r#"[toolchain]
channel = "1.44.66"
"#;
        assert_eq!(&contents, expected);

        let toolchain_file = tmp.path("rust-toolchain");
        assert!(!toolchain_file.exists()); // no ext variant should not exist
    }

    #[test]
    fn without_extension_takes_precedence() {
        let tmp = TestDir::temp()
            .create("rust-toolchain", FileType::EmptyFile)
            .create("rust-toolchain.toml", FileType::EmptyFile);

        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(1, 55, 77);

        let toolchain_file = tmp.path("rust-toolchain");
        let toolchain_file_toml = tmp.path("rust-toolchain.toml");

        let metadata = std::fs::metadata(&toolchain_file).unwrap(); // panics if file does not exist
        let metadata_toml = std::fs::metadata(&toolchain_file_toml).unwrap(); // panics if file does not exist
        assert_eq!(metadata.len(), 0); // created empty
        assert_eq!(metadata_toml.len(), 0); // created empty

        write_toolchain_file(&config, &fake_reporter, &version).unwrap();
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
        let tmp = TestDir::temp();
        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let test_reporter = TestReporterWrapper::default();
        let version = semver::Version::new(2, 0, 5);

        write_toolchain_file(&config, test_reporter.reporter(), &version).unwrap();

        let events = test_reporter.wait_for_events();
        let expected: Vec<Event> = vec![AuxiliaryOutput::new(
            Destination::file(tmp.path("rust-toolchain")),
            AuxiliaryOutputItem::toolchain_file(ToolchainFileKind::Toml),
        )
        .into()];

        phenomenon::contains_at_least_ordered(events, expected).assert_this();
    }

    #[test]
    fn write_failure() {
        let tmp = TestDir::temp().create("rust-toolchain", FileType::Dir); // dir so write will fail

        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(2, 0, 5);

        let error = write_toolchain_file(&config, &fake_reporter, &version).unwrap_err();
        let expected_path = tmp.path("rust-toolchain");

        assert!(matches!(
            error,
            CargoMSRVError::Io( IoError {
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
    use test_dir::{DirBuilder, FileType, TestDir};

    #[test]
    fn without_extension() {
        let tmp = TestDir::temp().create("rust-toolchain", FileType::EmptyFile);
        let crate_path = tmp.root();
        let toolchain_file = toolchain_file(crate_path);

        assert_eq!(toolchain_file, crate_path.join("rust-toolchain"));
    }

    #[test]
    fn with_extension() {
        let tmp = TestDir::temp().create("rust-toolchain.toml", FileType::EmptyFile);
        let crate_path = tmp.root();
        let toolchain_file = toolchain_file(crate_path);

        assert_eq!(toolchain_file, crate_path.join("rust-toolchain.toml"));
    }

    #[test]
    fn default() {
        let tmp = TestDir::temp();
        let crate_path = tmp.root();
        let toolchain_file = toolchain_file(crate_path);

        assert_eq!(toolchain_file, crate_path.join("rust-toolchain"));
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
