extern crate cargo_msrv;

use parameterized::parameterized;
use rust_releases::{semver, Release};

use cargo_msrv::MinimalCompatibility;
use common::{run_cargo_version_which_doesnt_support_lockfile_v2, run_msrv, run_msrv_with_releases};

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
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_msrv(with_args);
    let actual_version = result.unwrap_version();

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
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_msrv(with_args);
    let actual_version = result.unwrap_version();

    assert_eq!(actual_version, expected_version);
}

#[test]
fn msrv_unsupported() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join("unbuildable");
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_msrv(with_args);
    assert_eq!(result, MinimalCompatibility::NoCompatibleToolchains);
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
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec![
        "cargo-msrv",
        "--path",
        folder.to_str().unwrap(),
        "--",
        "cargo",
        "check",
    ];

    let result = run_msrv(with_args);
    let actual_version = result.unwrap_version();

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
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec![
        "cargo-msrv",
        "--release-source",
        release_source,
        "--path",
        folder.to_str().unwrap(),
        "--",
        "cargo",
        "check",
    ];

    let result = run_msrv(with_args);

    let actual_version = result.unwrap_version();

    assert_eq!(actual_version, expected_version);
}

#[test]
fn msrv_with_old_lockfile() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join("1.29.2");
    let with_args = vec![
        "cargo-msrv",
        "--path",
        folder.to_str().unwrap(),
        "--ignore-lockfile",
    ];

    let result = run_cargo_version_which_doesnt_support_lockfile_v2(with_args);
    assert_eq!(result.unwrap_version().minor, 29);
}

mod minimum_from_edition {
    use super::{Release, run_msrv_with_releases, semver};

    #[test]
    fn msrv_min_with_edition_in_cargo_toml() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("features")
            .join("1.30.0");
        let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

        let versions = vec![
            Release::new_stable(semver::Version::new(1, 32, 0)),
            Release::new_stable(semver::Version::new(1, 31, 0)),
            Release::new_stable(semver::Version::new(1, 30, 0)),
            Release::new_stable(semver::Version::new(1, 29, 0)),
        ];
        let (result, reporter) = run_msrv_with_releases(with_args, versions);
        assert_eq!(result.unwrap_version().minor, 31);
        assert_eq!(
            reporter.expose_successes(),
            vec![
                (true, semver::Version::new(1, 32, 0)),
                (true, semver::Version::new(1, 31, 0)),
            ]
        );
    }

    #[test]
    fn msrv_no_minimum_with_flag() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("features")
            .join("1.30.0");
        let with_args = vec![
            "cargo-msrv",
            "--path",
            folder.to_str().unwrap(),
            "--no-read-min-edition",
        ];

        let versions = vec![
            Release::new_stable(semver::Version::new(1, 32, 0)),
            Release::new_stable(semver::Version::new(1, 31, 0)),
            Release::new_stable(semver::Version::new(1, 30, 0)),
            Release::new_stable(semver::Version::new(1, 29, 0)),
        ];
        let (result, reporter) = run_msrv_with_releases(with_args, versions);
        assert_eq!(result.unwrap_version().minor, 31);
        assert_eq!(
            reporter.expose_successes(),
            vec![
                (true, semver::Version::new(1, 32, 0)),
                (true, semver::Version::new(1, 31, 0)),
                (false, semver::Version::new(1, 30, 0)),
            ]
        );
    }
}
