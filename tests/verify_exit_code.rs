use cargo_msrv::exit_code::ExitCode;
use std::process::Command;

#[test]
fn verify_success_zero_exit_code() {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");
    let test_subject = [cargo_msrv_dir, "features", "1.56.0-edition-2021"].join("/");

    let mut process = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--verify",
            "--path",
            &test_subject,
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Success;

    assert_eq!(exit_code, Into::<i32>::into(expected))
}

#[test]
fn verify_failure_non_zero_exit_code() {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");

    let test_subject = [cargo_msrv_dir, "features", "unbuildable-with-msrv"].join("/");

    let mut process = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--verify",
            "--path",
            &test_subject,
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Failure;

    assert_eq!(exit_code, Into::<i32>::into(expected))
}
