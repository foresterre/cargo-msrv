//! Generates the collection of stabilized standard library items, from the rustdoc JSON
//! of the standard library (the `rust-docs-json` rustup component, available on nightly).
//!
//! Usage: `cargo-msrv-std-since-generator <json-dir> <output-file> <toolchain>`, for example:
//!
//! ```shell
//! rustup component add rust-docs-json --toolchain nightly
//! cargo run --release -p cargo-msrv-std-since-generator -- \
//!     "$(rustc +nightly --print sysroot)/share/doc/rust/json" \
//!     crates/cargo-msrv-std-since/data/std_since.rkyv \
//!     "$(rustc +nightly -V)"
//! ```

mod json;
mod walk;

use std::collections::HashMap;
use std::path::Path;
use std::{env, fs, process};
use walk::{CRATES, Loaded, Walker};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let [json_dir, output, toolchain] = args.as_slice() else {
        eprintln!("usage: cargo-msrv-std-since-generator <json-dir> <output-file> <toolchain>");
        process::exit(2);
    };

    if let Err(message) = run(Path::new(json_dir), Path::new(output), toolchain) {
        eprintln!("error: {message}");
        process::exit(1);
    }
}

fn run(json_dir: &Path, output: &Path, toolchain: &str) -> Result<(), String> {
    // Items stabilized in the current release have `since = "CURRENT_RUSTC_VERSION"`.
    let current = toolchain
        .split_whitespace()
        .nth(1)
        .and_then(|version| version.split('-').next())
        .and_then(|version| version.parse().ok())
        .ok_or_else(|| format!("unable to find the version in toolchain '{toolchain}'"))?;

    let mut crates = HashMap::new();
    for name in CRATES {
        let path = json_dir.join(format!("{name}.json"));
        let contents = fs::read_to_string(&path)
            .map_err(|err| format!("unable to read '{}': {err}", path.display()))?;
        let krate = serde_json::from_str(&contents)
            .map_err(|err| format!("unable to parse '{}': {err}", path.display()))?;
        crates.insert(name, Loaded::new(krate));
    }

    let mut walked = Walker::new(&crates, current).walk();

    for version in &walked.unknown_versions {
        eprintln!("warning: skipped the items with the unknown version '{version}'");
    }

    walked.builder.set_toolchain(toolchain);

    let bytes = walked
        .builder
        .to_bytes()
        .map_err(|err| format!("unable to write the collection: {err}"))?;

    fs::write(output, bytes.as_slice())
        .map_err(|err| format!("unable to write '{}': {err}", output.display()))?;

    println!("wrote the collection to '{}'", output.display());

    Ok(())
}
