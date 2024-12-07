#![allow(clippy::items_after_test_module)]

extern crate cargo_msrv;

use cargo_msrv::error::CargoMSRVError;
use parameterized::parameterized;
use rust_releases::{semver, Release};
use std::path::Path;

use crate::common::sub_cmd_find::{
    find_msrv, find_msrv_with_releases, run_cargo_version_which_doesnt_support_lockfile_v2,
};
use crate::common::Fixture;

mod common;

// TODO can we enforce single threadness without macro's or external options like --test-threads=1

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
    let fixture = Fixture::new(folder);

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--linear",
        "--path",
        fixture.to_str(),
    ];

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
    let fixture = Fixture::new(folder);

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--bisect",
        "--path",
        fixture.to_str(),
    ];

    let result = find_msrv(with_args).unwrap();
    let actual_version = result.unwrap();

    assert_eq!(actual_version, expected_version);
}

#[test]
fn msrv_unsupported() {
    let fixture = Fixture::new("unbuildable");
    let with_args = vec!["cargo", "msrv", "find", "--path", fixture.to_str()];

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
    let fixture = Fixture::new(folder);

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--linear",
        "--path",
        fixture.to_str(),
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
    let fixture = Fixture::new(folder);

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--linear",
        "--release-source",
        release_source,
        "--path",
        fixture.to_str(),
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
    let fixture = Fixture::new("1.29.2");

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--linear",
        "--path",
        fixture.to_str(),
        "--ignore-lockfile",
    ];

    let result = run_cargo_version_which_doesnt_support_lockfile_v2(with_args).unwrap();
    assert_eq!(result.unwrap().minor, 29);
}

mod minimum_from_edition {
    use super::{semver, Release};
    use crate::common::sub_cmd_find::find_msrv_with_releases;
    use crate::common::Fixture;

    #[test]
    fn msrv_min_with_edition_in_cargo_toml() {
        let fixture = Fixture::new("1.30.0");

        let with_args = vec![
            "cargo",
            "msrv",
            "find",
            "--linear",
            "--path",
            fixture.to_str(),
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
                semver::Version::new(1, 31, 0)
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
    let fixture = Fixture::new(Path::new("virtual-workspace").join(package));
    let with_args = vec!["cargo", "msrv", "find", "--path", fixture.to_str()];

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();
    let actual_version = test_result.msrv().unwrap();

    assert_eq!(actual_version, &expected_version);
}

#[yare::parameterized(
    a_regular = {"cargo check", "a",  semver::Version::new(1,56,1) }, // `a` has an MSRV of 1.56.1
    b_regular = {"cargo check", "b", semver::Version::new(1,58,1) }, // `b` has an MSRV of 1.58.1
    a_workspace = {"cargo check --workspace", "a", semver::Version::new(1,58,1) }, // since `b` has a greater MSRV than `a`, the greatest common MSRV of the workspace is the MSRV of `b`: 1.58.1
    b_workspace = {"cargo check --workspace", "b", semver::Version::new(1,58,1)}, // the greatest common MSRV of the workspace is the MSRV of `b`: 1.58.1
)]
fn msrv_in_a_virtual_workspace(command: &str, package: &str, expected_version: semver::Version) {
    let fixture = Fixture::new("virtual-workspace");
    let package = fixture.tmp_path(package);

    let base_command = vec![
        "cargo",
        "msrv",
        "find",
        "--path",
        package.to_str().unwrap(),
        "--",
    ];
    let custom_check_command = command.split_ascii_whitespace().collect::<Vec<_>>();
    let command = [base_command, custom_check_command];

    let with_args = command.iter().flatten().collect::<Vec<_>>();

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();
    let actual_version = test_result.msrv().unwrap();

    assert_eq!(actual_version, &expected_version);
}

#[ignore = "TODO(#1081): 2024-11-30: Started failing on CI suddenly: cannot reproduce locally..."]
#[test]
fn cargo_features_option() {
    let fixture = Fixture::new("cargo-feature-required");

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--features",
        "required_feature",
        "--path",
        fixture.to_str(),
    ];

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();

    let version = test_result.msrv().unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 56);
    assert_eq!(
        test_result.successful_checks(),
        &[
            semver::Version::new(1, 58, 1),
            semver::Version::new(1, 56, 1)
        ]
    );
}

#[ignore = "TODO(#1081): 2024-11-30: Started failing on CI suddenly: cannot reproduce locally..."]
#[test]
fn cargo_all_features_flag() {
    let fixture = Fixture::new("cargo-feature-required");

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--all-features",
        "--path",
        fixture.to_str(),
    ];

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();

    let version = test_result.msrv().unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 56);
    assert_eq!(
        test_result.successful_checks(),
        &[
            semver::Version::new(1, 58, 1),
            semver::Version::new(1, 56, 1)
        ]
    );
}

#[ignore = "TODO(#1081): 2024-11-30: Started failing on CI suddenly: cannot reproduce locally..."]
#[test]
fn cargo_no_default_features_flag() {
    let fixture = Fixture::new("cargo-feature-requires-none");

    let with_args = vec![
        "cargo",
        "msrv",
        "find",
        "--no-default-features",
        "--path",
        fixture.to_str(),
    ];

    let versions = vec![
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 56, 1)),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();

    let version = test_result.msrv().unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 56);
    assert_eq!(
        test_result.successful_checks(),
        &[
            semver::Version::new(1, 58, 1),
            semver::Version::new(1, 56, 1)
        ]
    );
}
