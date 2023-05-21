use crate::context::{EnvironmentContext, RustReleasesContext, SetContext, UserOutputContext};
use crate::manifest::bare_version::BareVersion;
use crate::reporter::Reporter;
use crate::{Set, SubCommand, TResult};
use rust_releases::ReleaseIndex;

/// Write the MSRV to the Cargo manifest
///
/// Repurposes the Set MSRV subcommand for this action.
pub fn write_msrv(
    reporter: &impl Reporter,
    msrv: BareVersion,
    release_index: Option<&ReleaseIndex>,
    environment: EnvironmentContext,
    user_output: UserOutputContext,
    rust_releases: RustReleasesContext,
) -> TResult<()> {
    let context = SetContext {
        msrv,
        environment,
        user_output,
        rust_releases,
    };

    Set::new(release_index).run(&context, reporter)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::context::{EnvironmentContext, RustReleasesContext, UserOutputContext};
    use crate::error::CargoMSRVError;
    use crate::manifest::bare_version::BareVersion;
    use crate::reporter::FakeTestReporter;
    use crate::writer::write_msrv::write_msrv;
    use crate::OutputFormat;
    use camino::Utf8Path;
    use rust_releases::{semver, ReleaseIndex};
    use std::iter::FromIterator;
    use test_dir::{DirBuilder, FileType, TestDir};

    #[test]
    fn set_release_in_index() {
        let tmp = TestDir::temp().create("Cargo.toml", FileType::EmptyFile);
        let manifest = tmp.path("Cargo.toml");
        std::fs::write(&manifest, "[package]").unwrap();

        let root = tmp.root();
        let root = Utf8Path::from_path(root).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = BareVersion::ThreeComponents(2, 0, 5);

        let env = EnvironmentContext {
            crate_path: root.to_path_buf(),
        };

        let user_output = UserOutputContext {
            output_format: OutputFormat::None,
        };

        let index = ReleaseIndex::from_iter(vec![rust_releases::Release::new_stable(
            semver::Version::new(2, 0, 5),
        )]);

        write_msrv(
            &fake_reporter,
            version,
            Some(&index),
            env,
            user_output,
            RustReleasesContext::default(),
        )
        .unwrap();

        let content = std::fs::read_to_string(&manifest).unwrap();
        assert_eq!(content, "[package]\nrust-version = \"2.0.5\"\n");
    }

    #[test]
    fn fail_to_set_release_not_in_index() {
        let tmp = TestDir::temp().create("Cargo.toml", FileType::EmptyFile);
        let manifest = tmp.path("Cargo.toml");
        std::fs::write(manifest, "[package]").unwrap();

        let root = tmp.root();
        let root = Utf8Path::from_path(root).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = BareVersion::ThreeComponents(2, 0, 5);

        let env = EnvironmentContext {
            crate_path: root.to_path_buf(),
        };

        let user_output = UserOutputContext {
            output_format: OutputFormat::None,
        };

        let index = ReleaseIndex::from_iter(vec![]);

        let err = write_msrv(
            &fake_reporter,
            version,
            Some(&index),
            env,
            user_output,
            RustReleasesContext::default(),
        )
        .unwrap_err();

        assert!(matches!(err, CargoMSRVError::InvalidMsrvSet(_)));
    }

    #[test]
    fn set_release_without_index_check() {
        let tmp = TestDir::temp().create("Cargo.toml", FileType::EmptyFile);
        let manifest = tmp.path("Cargo.toml");
        std::fs::write(&manifest, "[package]").unwrap();

        let root = tmp.root();
        let root = Utf8Path::from_path(root).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = BareVersion::ThreeComponents(2, 0, 5);

        let env = EnvironmentContext {
            crate_path: root.to_path_buf(),
        };

        let user_output = UserOutputContext {
            output_format: OutputFormat::None,
        };

        write_msrv(
            &fake_reporter,
            version,
            None,
            env,
            user_output,
            RustReleasesContext::default(),
        )
        .unwrap();

        let content = std::fs::read_to_string(&manifest).unwrap();
        assert_eq!(content, "[package]\nrust-version = \"2.0.5\"\n");
    }
}
