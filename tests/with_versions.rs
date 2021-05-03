extern crate cargo_msrv;

use cargo_msrv::MinimalCompatibility;
use parameterized::parameterized;
use rust_releases::{semver, Release, ReleaseIndex};
use std::ffi::OsString;
use std::iter::FromIterator;

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
    let with_args = vec!["cargo", "msrv", "--path", folder.to_str().unwrap()];

    let result = run(with_args);

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
    let with_args = vec!["cargo", "msrv", "--path", folder.to_str().unwrap()];

    let result = run(with_args);

    let actual_version = result.unwrap_version();

    assert_eq!(actual_version, expected_version);
}

#[test]
fn msrv_unsupported() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join("unbuildable");
    let with_args = vec!["cargo", "msrv", "--path", folder.to_str().unwrap()];

    let result = run(with_args);
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
        "cargo",
        "msrv",
        "--path",
        folder.to_str().unwrap(),
        "--",
        "cargo",
        "check",
    ];

    let result = run(with_args);

    let actual_version = result.unwrap_version();

    assert_eq!(actual_version, expected_version);
}

fn run<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(with_args: I) -> MinimalCompatibility {
    let matches = cargo_msrv::cli::cli().get_matches_from(with_args);
    let matches = cargo_msrv::cli::cmd_matches(&matches).expect("Unable to parse cli arguments");

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions: ReleaseIndex = FromIterator::from_iter(vec![
        Release::new_stable(semver::Version::new(1, 38, 0)),
        Release::new_stable(semver::Version::new(1, 37, 0)),
        Release::new_stable(semver::Version::new(1, 36, 0)),
        Release::new_stable(semver::Version::new(1, 35, 0)),
        Release::new_stable(semver::Version::new(1, 34, 0)),
    ]);

    // Determine the MSRV from the index of available releases.
    cargo_msrv::determine_msrv(&matches, &available_versions).expect("Unable to run MSRV process")
}
