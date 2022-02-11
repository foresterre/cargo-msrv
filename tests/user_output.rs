use std::path::Path;
use std::process::{Command, Stdio};

use crate::common::fixtures_path;

mod common;

#[test]
fn expect_no_user_output() {
    let cargo_msrv_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let test_subject = fixtures_path().join("unbuildable-with-msrv");

    let process = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--manifest-path",
            &cargo_msrv_manifest.to_str().unwrap(),
            "--",
            "--path",
            test_subject.to_str().unwrap(),
            "--no-user-output", // this is the command we're testing
            "verify",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let output = process
        .wait_with_output()
        .expect("Waiting for process failed during test");

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Unable to test for this because of: https://github.com/foresterre/cargo-msrv/issues/263
    // assert!(stdout.is_empty()); // FIXME(foresterre): #263
    // assert!(stderr.is_empty()); // FIXME(foresterre): #263
}
