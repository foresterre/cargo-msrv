// This module mirrors the tests in 'find_msrv' in the same folder. Between that test module
// and this one, there is one difference: that module calls `cargo msrv find` while this one
// calls just `cargo msrv`, the top level command without the `find` subcommand,
// which is still supported for backwards compatibility.
// When we say mirror, currently, we simply mean copied :).
extern crate cargo_msrv;

use crate::common::fixture::copy_fixture_to_test_dir;
use cargo_msrv::error::CargoMSRVError;
use parameterized::parameterized;
use rust_releases::{semver, Release};

use crate::common::sub_cmd_find::{
    find_msrv, find_msrv_with_releases, run_cargo_version_which_doesnt_support_lockfile_v2,
};

mod common;

#[parameterized(
    folder = {
        "1.35.0",
        "1.36.0",
        "1.37.0",
        "1.38.0",
    },
    expected_version = {
        semver::Version::new(1,35,0),
        semver::Version::new(1,36,0),
        semver::Version::new(1,37,0),
        semver::Version::new(1,38,0),
    }
)]
fn msrv_using_linear_method(folder: &str, expected_version: semver::Version) {
    let folder = copy_fixture_to_test_dir(folder);

    let with_args = vec!["cargo", "msrv", "--linear", "--path", folder.path_as_str()];

    let result = find_msrv(with_args).unwrap();
    let actual_version = result.unwrap();

    assert_eq!(actual_version, expected_version);
}

#[parameterized(
    folder = {
        "1.35.0",
        "1.36.0",
        "1.37.0",
        "1.38.0",
    },
    expected_version = {
        semver::Version::new(1,35,0),
        semver::Version::new(1,36,0),
        semver::Version::new(1,37,0),
        semver::Version::new(1,38,0),
    }
)]
fn msrv_using_bisect_method(folder: &str, expected_version: semver::Version) {
    let folder = copy_fixture_to_test_dir(folder);

    let with_args = vec!["cargo", "msrv", "--bisect", "--path", folder.path_as_str()];

    let result = find_msrv(with_args).unwrap();
    let actual_version = result.unwrap();

    assert_eq!(actual_version, expected_version);
}

#[test]
fn msrv_unsupported() {
    let folder = copy_fixture_to_test_dir("unbuildable");

    let with_args = vec!["cargo", "msrv", "--path", folder.path_as_str()];

    let result = find_msrv(with_args);

    assert!(matches!(
        result.unwrap_err(),
        CargoMSRVError::UnableToFindAnyGoodVersion { .. }
    ));
}

#[parameterized(
    folder = {
        "1.35.0",
        "1.36.0",
        "1.37.0",
        "1.38.0",
    },
    expected_version = {
        semver::Version::new(1,35,0),
        semver::Version::new(1,36,0),
        semver::Version::new(1,37,0),
        semver::Version::new(1,38,0),
    }
)]
fn msrv_with_custom_command(folder: &str, expected_version: semver::Version) {
    let folder = copy_fixture_to_test_dir(folder);

    let with_args = vec![
        "cargo",
        "msrv",
        "--linear",
        "--path",
        folder.path_as_str(),
        "--",
        "cargo",
        "check",
    ];

    let result = find_msrv(with_args).unwrap();
    let actual_version = result.unwrap();

    assert_eq!(actual_version, expected_version);
}

#[parameterized(
    release_source = {
        "rust-changelog",
        "rust-dist"
    },
    folder = {
        "1.38.0",
        "1.38.0"
    },
    expected_version = {
        semver::Version::new(1,38,0),
        semver::Version::new(1,38,0),
    }
)]
fn msrv_with_release_source(release_source: &str, folder: &str, expected_version: semver::Version) {
    let folder = copy_fixture_to_test_dir(folder);

    let with_args = vec![
        "cargo",
        "msrv",
        "--linear",
        "--release-source",
        release_source,
        "--path",
        folder.path_as_str(),
        "--",
        "cargo",
        "check",
    ];

    let result = find_msrv(with_args).unwrap();

    let actual_version = result.unwrap();

    assert_eq!(actual_version, expected_version);
}

#[test]
fn msrv_with_old_lockfile() {
    let folder = copy_fixture_to_test_dir("1.29.2");
    let with_args = vec![
        "cargo",
        "msrv",
        "--linear",
        "--path",
        folder.path_as_str(),
        "--ignore-lockfile",
    ];

    let result = run_cargo_version_which_doesnt_support_lockfile_v2(with_args).unwrap();
    assert_eq!(result.unwrap().minor, 29);
}

mod minimum_from_edition {
    use super::{semver, Release};
    use crate::common::fixture::copy_fixture_to_test_dir;
    use crate::common::sub_cmd_find::find_msrv_with_releases;

    #[test]
    fn msrv_min_with_edition_in_cargo_toml() {
        let folder = copy_fixture_to_test_dir("1.30.0");

        let with_args = vec!["cargo", "msrv", "--linear", "--path", folder.path_as_str()];

        let versions = vec![
            Release::new_stable(semver::Version::new(1, 32, 0)),
            Release::new_stable(semver::Version::new(1, 31, 0)),
            Release::new_stable(semver::Version::new(1, 30, 0)),
            Release::new_stable(semver::Version::new(1, 29, 0)),
        ];

        let test_result = find_msrv_with_releases(with_args, versions).unwrap();

        assert_eq!(test_result.msrv().unwrap().minor, 31);
        assert_eq!(
            test_result.successful_checks(),
            &[
                semver::Version::new(1, 32, 0),
                semver::Version::new(1, 31, 0)
            ]
        );
    }

    #[test]
    fn msrv_no_minimum_with_flag() {
        let folder = copy_fixture_to_test_dir("1.30.0");

        let with_args = vec![
            "cargo",
            "msrv",
            "--linear",
            "--path",
            folder.path_as_str(),
            "--no-read-min-edition",
        ];

        let versions = vec![
            Release::new_stable(semver::Version::new(1, 32, 0)),
            Release::new_stable(semver::Version::new(1, 31, 0)),
            Release::new_stable(semver::Version::new(1, 30, 0)),
            Release::new_stable(semver::Version::new(1, 29, 0)),
        ];

        let test_result = find_msrv_with_releases(with_args, versions).unwrap();

        assert_eq!(test_result.msrv().unwrap().minor, 31);
        assert_eq!(
            test_result.successful_checks(),
            &[
                semver::Version::new(1, 32, 0),
                semver::Version::new(1, 31, 0),
                semver::Version::new(1, 30, 0),
            ]
        );
    }
}
//
#[parameterized(
    package = {
        "a",
        "b",
    },
    expected_version = {
        semver::Version::new(1,56,1), // `a` has an MSRV of 1.56.1
        semver::Version::new(1,58,1), // `b` has an MSRV of 1.58.1
    }
)]
fn msrv_in_a_virtual_workspace_default_check_command(
    package: &str,
    expected_version: semver::Version,
) {
    let folder = copy_fixture_to_test_dir("virtual-workspace");
    let folder = folder.path().join(package);
    let folder = folder.to_str().unwrap();

    let with_args = vec!["cargo", "msrv", "--path", folder];

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();
    let actual_version = test_result.msrv().unwrap();

    assert_eq!(actual_version, &expected_version);
}

#[parameterized(
    command = {
        "cargo check",
        "cargo check --workspace",
        "cargo check",
        "cargo check --workspace",
    },
    package = {
        "a",
        "a",
        "b",
        "b",
    },
    expected_version = {
        semver::Version::new(1,56,1), // `a` has an MSRV of 1.56.1
        semver::Version::new(1,58,1), // since `b` has a greater MSRV than `a`, the greatest common MSRV of the workspace is the MSRV of `b`: 1.58.1
        semver::Version::new(1,58,1), // `b` has an MSRV of 1.58.1
        semver::Version::new(1,58,1), // the greatest common MSRV of the workspace is the MSRV of `b`: 1.58.1
    }
)]
fn msrv_in_a_virtual_workspace(command: &str, package: &str, expected_version: semver::Version) {
    let folder = copy_fixture_to_test_dir("virtual-workspace");
    let folder = folder.path().join(package);
    let folder = folder.to_str().unwrap();

    let base_command = vec!["cargo", "msrv", "--path", folder, "--"];
    let custom_check_command = command.split_ascii_whitespace().collect::<Vec<_>>();
    let command = vec![base_command, custom_check_command];

    let with_args = command.iter().flatten().collect::<Vec<_>>();

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();
    let actual_version = test_result.msrv().unwrap();

    assert_eq!(actual_version, &expected_version);
}
