#![allow(clippy::items_after_test_module)]

use cargo_msrv::exit_code::ExitCode;
use parameterized::parameterized;
use rust_releases::{semver, Release};
use std::process::Command;

use crate::common::{sub_cmd_verify::run_verify, Fixture};

mod common;

#[parameterized(
    folder = {
        "1.35.0",
        "1.36.0",
        "1.56.0-edition-2018",
        "1.56.0-edition-2021",
    }
)]
fn verify(folder: &str) {
    let fixture = Fixture::new(folder);
    let with_args = vec![
        "cargo",
        "msrv",
        "--path",
        fixture.to_str(),
        "--no-user-output",
        "verify",
    ];

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

#[yare::parameterized(
    a = {"workspace-inheritance", "a", },
    b = { "workspace-inheritance", "b" },
    c = { "workspace-inheritance", "c" },
)]
fn verify_workspace_inheritance(folder: &str, package: &str) {
    let fixture = Fixture::new(folder);
    let package = fixture.tmp_path(package);

    let with_args = vec![
        "cargo",
        "msrv",
        "--path",
        package.to_str().unwrap(),
        "--no-user-output",
        "verify",
    ];

    let result = run_verify(
        with_args,
        vec![
            // only stabilized in 1.64.0
            Release::new_stable(semver::Version::new(1, 64, 0)),
            Release::new_stable(semver::Version::new(1, 66, 0)),
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
    let fixture = Fixture::new(folder);
    let with_args = vec!["cargo", "msrv", "--path", fixture.to_str(), "verify"];

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

#[parameterized(
    verify_variant = {
        "verify", // as sub command after options and flags
    }
)]
fn verify_success_zero_exit_code(verify_variant: &str) {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");
    let test_subject = Fixture::new("1.56.0-edition-2021");

    let mut process = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--no-user-output",
            "--path",
            test_subject.to_str(),
            verify_variant,
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Success;

    assert_eq!(exit_code, Into::<i32>::into(expected));
}

#[parameterized(
    verify_variant = {
        "verify", // as sub command after options and flags
    }
)]
fn verify_failure_non_zero_exit_code(verify_variant: &str) {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");
    let test_subject = Fixture::new("unbuildable-with-msrv");

    let mut process = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--no-user-output",
            "--path",
            test_subject.to_str(),
            verify_variant,
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Failure;

    assert_eq!(exit_code, Into::<i32>::into(expected));
}

#[test]
fn verify_subcommand_success_with_custom_check_cmd() {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");
    let test_subject = Fixture::new("1.56.0-edition-2021");

    let mut process = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--no-user-output",
            "--path",
            test_subject.to_str(),
            "verify",
            "--",
            "cargo",
            "build",
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Success;

    assert_eq!(exit_code, Into::<i32>::into(expected));
}

#[test]
fn verify_with_rust_version_opt() {
    let version = "1.37.0";
    let fixture = Fixture::new(version);

    let with_args = vec![
        "cargo",
        "msrv",
        "--path",
        fixture.to_str(),
        "verify",
        "--rust-version",
        version,
    ];

    let result = run_verify(
        with_args,
        vec![Release::new_stable(semver::Version::new(1, 37, 0))],
    );

    assert!(result.is_ok());
}

#[test]
fn manifest_path() {
    let fixture = Fixture::new("1.36.0");
    let manifest = fixture.tmp_path("Cargo.toml");

    let with_args = vec![
        "cargo",
        "msrv",
        "--manifest-path",
        manifest.to_str().unwrap(),
        "verify",
    ];

    let result = run_verify(
        with_args,
        vec![Release::new_stable(semver::Version::new(1, 36, 0))],
    );

    assert!(result.is_ok());
}
