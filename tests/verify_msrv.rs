use rust_releases::{semver, Release};

mod common;

use common::*;
use parameterized::parameterized;

#[parameterized(
    folder = {
        "1.35.0",
        "1.36.0",
        "1.56.0-edition-2018",
        "1.56.0-edition-2021",
    }
)]
fn verify(folder: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_verify(
        with_args,
        vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 37, 0)),
            Release::new_stable(semver::Version::new(1, 36, 0)),
            Release::new_stable(semver::Version::new(1, 35, 0)),
            Release::new_stable(semver::Version::new(1, 34, 0)),
        ],
    );

    assert!(result.is_ok());
}

#[parameterized(
    folder = {
        "1.37.0",
        "1.38.0",
    }
)]
fn verify_failed_no_msrv_specified(folder: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_verify(
        with_args,
        vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 37, 0)),
            Release::new_stable(semver::Version::new(1, 36, 0)),
            Release::new_stable(semver::Version::new(1, 35, 0)),
            Release::new_stable(semver::Version::new(1, 34, 0)),
        ],
    );

    assert!(result.is_err());
}
