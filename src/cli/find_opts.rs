use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use clap::Args;

// Cli Options for top-level cargo-msrv (find) command
#[derive(Debug, Args)]
#[command(next_help_heading = "Find MSRV options")]
pub struct FindOpts {
    /// Use a binary search to find the MSRV (default)
    ///
    /// When the search space is sufficiently large, which is common, this is much
    /// faster than a linear search. A binary search will approximately halve the search
    /// space for each Rust version checked for compatibility.
    #[arg(long, conflicts_with = "linear")]
    pub bisect: bool,

    /// Use a linear search to find the MSRV
    ///
    /// This method checks toolchain from the most recent release to the earliest.
    #[arg(long, conflicts_with = "bisect")]
    pub linear: bool,

    /// Pin the MSRV by writing the version to a rust-toolchain file
    ///
    /// The toolchain file will pin the Rust version for this crate.
    /// See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more.
    #[arg(long, alias = "toolchain-file")]
    pub write_toolchain_file: bool,

    /// Temporarily remove the lockfile, so it will not interfere with the building process
    ///
    /// This is important when testing against older Rust versions such as Cargo versions prior to
    /// Rust 1.38.0, for which Cargo does not recognize the newer lockfile formats.
    #[arg(long)]
    pub ignore_lockfile: bool,

    /// Don't print the result of compatibility checks
    ///
    /// The feedback of a compatibility check can be useful to determine why a certain Rust
    /// version is not compatible. Rust usually prints very detailed error messages.
    /// While most often very useful, in some cases they may be too noisy or lengthy.
    /// If this flag is given, the result messages will not be printed.
    #[arg(long)]
    pub no_check_feedback: bool,

    /// Write the MSRV to the Cargo manifest
    ///
    /// For toolchains which include a Cargo version which supports the rust-version field,
    /// the `package.rust-version` field will be written. For older Rust toolchains,
    /// the `package.metadata.msrv` field will be written instead.
    #[arg(long)]
    pub write_msrv: bool,

    #[command(flatten)]
    pub rust_releases_opts: RustReleasesOpts,

    #[command(flatten)]
    pub toolchain_opts: ToolchainOpts,

    #[command(flatten)]
    pub custom_check_opts: CustomCheckOpts,
}
