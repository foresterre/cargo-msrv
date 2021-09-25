use std::path::{Path, PathBuf};

use crate::{errors::TResult, fetch::is_target_available};
use clap::Clap;
use rust_releases::semver::Version;

#[derive(Clap)]
pub struct Config {
    /// Path to the cargo project directory
    #[clap(long = "path", validator = "path_exists_validator", value_name = "DIR")]
    seek_path: Option<PathBuf>,

    /// Check against a custom target (instead of the rustup default)
    #[clap(
        long = "target",
        validator = "seek_target_validator",
        value_name = "TARGET"
    )]
    seek_target: Option<String>,

    /// Include all patch releases, instead of only the last
    #[clap(long)]
    include_all_patch_releases: bool,

    /// Earliest version to take into account
    ///
    /// Version must match a valid Rust toolchain, and be semver compatible.
    #[clap(long, alias = "minimum")]
    min: Option<Version>,

    /// Latest version to take into account
    ///
    /// Version must match a valid Rust toolchain, and be semver compatible.
    #[clap(long, alias = "maximum")]
    max: Option<Version>,

    /// Use a binary search to find the MSRV instead of a linear search
    #[clap(long)]
    bisect: bool,

    /// Output a rust-toolchain file with the MSRV as toolchain
    ///
    /// The toolchain file will pin the Rust version for this crate.
    /// See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more.
    #[clap(long)]
    toolchain_file: bool,

    /// Temporarily removes the lockfile, so it will not interfere with the building process
    ///
    /// This is important when testing against Rust versions prior to 1.38.0, for which Cargo does not recognize the new v2 lockfile.
    #[clap(long)]
    ignore_lockfile: bool,

    /// Output status messages in machine-readable format
    ///
    /// Machine-readable status updates will be printed in the requested format to stdout.
    #[clap(long, default_value = "human")]
    output_format: OutputFormat,

    /// Verify the MSRV, if defined with the 'package.metadata.msrv' key in the Cargo.toml
    ///
    /// When this flag is present, cargo-msrv will not attempt to determine the true MSRV.
    /// It will only attempt to verify specified MSRV, the Rust build passes similarly to regular cargo-msrv runs.
    #[clap(long)]
    verify: bool,

    /// If given, this command is used to validate if a Rust version is
    /// compatible. Should be available to rustup, i.e. the command should work like
    /// so: `rustup run <toolchain> <COMMAND>`.
    /// The default check action is `cargo check --all`.
    #[clap(setting(clap::ArgSettings::Last))]
    custom_command: Vec<String>,
}

fn path_exists_validator(path: &Path) -> Result<(), String> {
    std::fs::metadata(path)
        .map_err(|_| "Path doesn't exist.".to_string())
        .and_then(|m| {
            if m.is_dir() {
                Ok(())
            } else {
                Err("Not a directory.".to_string())
            }
        })
}

fn seek_target_validator(target: &str) -> TResult<()> {
    is_target_available(target).map_err(|_| {
        "The provided target is not available. Use `rustup target list` to \
             review the available targets."
            .to_string()
    })
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Progress bar rendered to stderr
    Human,
    /// Json status updates printed to stdout
    Json,
    /// No output -- meant to be used for testing
    None,
}
