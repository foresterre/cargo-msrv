use std::path::Path;
use std::process::{Command, Stdio};

use crate::common::fixtures_path;

mod common;

#[test]
fn expect_no_user_output() {
    let cargo_msrv_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let test_subject = fixtures_path().join("1.36.0");

    let process = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--manifest-path",
            cargo_msrv_manifest.to_str().unwrap(),
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The empty string, "", is preferred over is_empty() because if the assertion fails, we'll
    // see the diff between the expected and actual strings.
    assert_eq!(stdout.as_ref(), "");
    assert_eq!(stderr.as_ref(), "");
}
