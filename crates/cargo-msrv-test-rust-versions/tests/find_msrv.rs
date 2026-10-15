use assert_fs::TempDir;
use assert_fs::fixture::PathCopy;
use cargo_metadata::Message;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[yare::parameterized(
    v1_90_0 = { "1.90.0" },
    v1_91_1 = { "1.91.1" },
    v1_92_0 = { "1.92.0" },
    v1_93_1 = { "1.93.1" },
    v1_94_1 = { "1.94.1" },
    v1_95_0 = { "1.95.0" },
    v1_96_1 = { "1.96.1" },
    v1_97_1 = { "1.97.1" },
    v1_98_1 = { "1.98.1" },
    v1_99_0 = { "1.99.0" },
)]
#[cfg_attr(
    not(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")),
    ignore = "older toolchains are only available on x86_64-unknown-linux-gnu"
)]
fn msrv_using_bisect_method_on_rust_versions(version: &str) {
    let fixture = TempDir::new().unwrap();
    fixture
        .copy_from(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join(version),
            &["**/*"],
        )
        .unwrap();

    let output = Command::new(cargo_msrv())
        .args([
            "msrv",
            "find",
            "--bisect",
            "--release-source",
            "bundled",
            "--output-format",
            "minimal",
            "--no-log",
            "--path",
        ])
        .arg(fixture.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(stdout.trim(), version, "stderr:\n{stderr}");
    assert!(output.status.success(), "stderr:\n{stderr}");
}

fn cargo_msrv() -> &'static Path {
    static CARGO_MSRV: OnceLock<PathBuf> = OnceLock::new();

    CARGO_MSRV.get_or_init(build_cargo_msrv)
}

fn build_cargo_msrv() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.toml");

    let mut build = Command::new(env!("CARGO"))
        .args([
            "build",
            "--package",
            "cargo-msrv",
            "--bin",
            "cargo-msrv",
            "--message-format=json-render-diagnostics",
            "--manifest-path",
        ])
        .arg(manifest)
        .stdout(Stdio::piped())
        .spawn()
        .expect("unable to start building cargo-msrv");

    let stdout = build.stdout.take().unwrap();
    let executable = Message::parse_stream(BufReader::new(stdout))
        .filter_map(|message| match message.unwrap() {
            Message::CompilerArtifact(artifact) if artifact.target.name == "cargo-msrv" => {
                artifact.executable
            }
            _ => None,
        })
        .last();

    let status = build.wait().unwrap();
    assert!(status.success(), "unable to build cargo-msrv");

    executable
        .expect("unable to find the cargo-msrv binary")
        .into_std_path_buf()
}
