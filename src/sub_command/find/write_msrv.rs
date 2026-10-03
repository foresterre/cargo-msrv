use crate::Set;
use crate::TResult;
use crate::reporter::Reporter;
use crate::sub_command::set::set_msrv;
use camino::Utf8Path;
use cargo_msrv_context::SelectedPackage;
use cargo_msrv_rust_releases::ReleaseIndex;
use cargo_msrv_types::BareVersion;

/// Write the MSRV to the Cargo manifest
///
/// Repurposes the Set MSRV subcommand for this action.
pub fn write_msrv(
    reporter: &impl Reporter,
    msrv: BareVersion,
    release_index: Option<&ReleaseIndex>,
    cargo_toml: &Utf8Path,
    package: Option<SelectedPackage>,
) -> TResult<()> {
    Set::new(release_index).check_release(&msrv, reporter)?;
    set_msrv(reporter, cargo_toml, package, &msrv)
}

#[cfg(test)]
mod tests {
    use crate::error::CargoMSRVError;
    use crate::reporter::FakeTestReporter;
    use crate::sub_command::find::write_msrv::write_msrv;
    use assert_fs::prelude::*;
    use camino::Utf8Path;
    use cargo_msrv_rust_releases::{ReleaseIndex, RustRelease, Stable};
    use cargo_msrv_types::BareVersion;
    use std::iter::FromIterator;

    #[test]
    fn set_release_in_index() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("Cargo.toml").touch().unwrap();

        let manifest = tmp.join("Cargo.toml");
        std::fs::write(&manifest, "[package]").unwrap();

        let manifest_path = Utf8Path::from_path(&manifest).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = BareVersion::ThreeComponents(2, 0, 5);

        let index = ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(2, 0, 5), None, [])]);

        write_msrv(&fake_reporter, version, Some(&index), manifest_path, None).unwrap();

        let content = std::fs::read_to_string(&manifest).unwrap();
        assert_eq!(content, "[package]\nrust-version = \"2.0.5\"\n");
    }

    #[test]
    fn fail_to_set_release_not_in_index() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("Cargo.toml").touch().unwrap();

        let manifest = tmp.join("Cargo.toml");
        std::fs::write(&manifest, "[package]").unwrap();

        let manifest_path = Utf8Path::from_path(&manifest).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = BareVersion::ThreeComponents(2, 0, 5);

        let index = ReleaseIndex::from_iter(vec![]);

        let err =
            write_msrv(&fake_reporter, version, Some(&index), manifest_path, None).unwrap_err();

        assert!(matches!(err, CargoMSRVError::InvalidMsrvSet(_)));
    }

    #[test]
    fn set_release_without_index_check() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("Cargo.toml").touch().unwrap();

        let manifest = tmp.join("Cargo.toml");
        std::fs::write(&manifest, "[package]").unwrap();

        let manifest_path = Utf8Path::from_path(&manifest).unwrap();

        let fake_reporter = FakeTestReporter::default();
        let version = BareVersion::ThreeComponents(2, 0, 5);

        write_msrv(&fake_reporter, version, None, manifest_path, None).unwrap();

        let content = std::fs::read_to_string(&manifest).unwrap();
        assert_eq!(content, "[package]\nrust-version = \"2.0.5\"\n");
    }
}
