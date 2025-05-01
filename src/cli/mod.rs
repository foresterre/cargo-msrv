use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use crate::context::list::ListMsrvVariant;
use crate::manifest::bare_version::BareVersion;
use clap::{Args, Parser, Subcommand};
use clap_cargo::style::CLAP_STYLING;
use std::ffi::{OsStr, OsString};

pub(crate) mod custom_check_opts;
pub(crate) mod rust_releases_opts;
pub(crate) mod shared_opts;
pub(crate) mod toolchain_opts;

#[derive(Debug, Parser)]
#[command(version, name = "cargo", bin_name = "cargo", max_term_width = 120, styles = CLAP_STYLING)]
pub struct CargoCli {
    #[command(subcommand)]
    subcommand: CargoMsrvCli,
}

impl CargoCli {
    pub fn parse_args<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(args: I) -> Self {
        let modified_args = modify_args(args);
        CargoCli::parse_from(modified_args)
    }

    pub fn to_cargo_msrv_cli(self) -> CargoMsrvCli {
        self.subcommand
    }
}

// When we call cargo-msrv with cargo, cargo will supply the msrv subcommand, in addition
// to the binary name itself. As a result, when you call cargo-msrv without cargo, for example
// `cargo-msrv` (without cargo) instead of `cargo msrv` (with cargo), the process will receive
// too many arguments, and you will have to specify the subcommand again like so: `cargo-msrv msrv`.
// This function removes the subcommand when it's present in addition to the program name.
fn modify_args<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(
    args: I,
) -> impl IntoIterator<Item = OsString> {
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();

    if args.len() >= 2 {
        let program: &OsStr = args[0].as_os_str();
        let program = program.to_string_lossy();

        // when `cargo-msrv(.exe)` or `msrv` are present
        if program.ends_with("cargo-msrv") || program.ends_with("cargo-msrv.exe") {
            // remove `msrv`, or `cargo-msrv(.exe)`
            args[0] = OsStr::new("cargo").to_os_string();

            let cargo_msrv_subcmd: &OsStr = args[1].as_os_str();
            let cargo_msrv_subcmd = cargo_msrv_subcmd.to_string_lossy();

            if cargo_msrv_subcmd != "msrv" {
                args.insert(1, OsStr::new("msrv").to_os_string());
            }
        }
    }

    args
}

#[derive(Debug, Subcommand)]
pub enum CargoMsrvCli {
    /// Find your Minimum Supported Rust Version!
    #[command(
        author = "Martijn Gribnau <garm@ilumeo.com>",
        after_help = r#"
            You can provide a custom compatibility check command as the last positional argument via
            the -- syntax, e.g. `$ cargo msrv find -- my custom command`.

            This custom check command will then be used to validate whether a Rust version is
            compatible.

            A custom `check` command should be runnable by rustup, as it will be passed to
            rustup like so: `rustup run <toolchain> <COMMAND...>`.
            NB: You only need to provide the <COMMAND...> part.

            By default, the compatibility check command is `cargo check`.
        "#
    )]
    Msrv(CargoMsrvOpts),
}

impl CargoMsrvCli {
    pub fn to_opts(self) -> CargoMsrvOpts {
        match self {
            Self::Msrv(opts) => opts,
        }
    }
}

#[derive(Debug, Args)]
#[command(version)]
pub struct CargoMsrvOpts {
    #[command(flatten)]
    pub shared_opts: SharedOpts,

    #[command(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Subcommand)]
#[command(propagate_version = true)]
pub enum SubCommand {
    /// Find the MSRV
    Find(FindOpts),
    /// Display the MSRV's of dependencies
    List(ListOpts),
    /// Set the MSRV of the current crate to a given Rust version
    Set(SetOpts),
    /// Show the MSRV of your crate, as specified in the Cargo manifest
    Show,
    /// Verify whether the MSRV is satisfiable.
    ///
    ///  The MSRV must be specified via the `--rust-version` option, or via the 'package.rust-version' or 'package.metadata.msrv' keys in the Cargo.toml manifest.
    Verify(VerifyOpts),
}

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
    #[arg(long, visible_alias = "set")]
    pub write_msrv: bool,

    #[command(flatten)]
    pub rust_releases_opts: RustReleasesOpts,

    #[command(flatten)]
    pub toolchain_opts: ToolchainOpts,

    #[command(flatten)]
    pub custom_check_opts: CustomCheckOpts,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "List options")]
pub struct ListOpts {
    /// Display the MSRV's of crates that your crate depends on
    #[arg(long, value_enum, default_value_t)]
    pub variant: ListMsrvVariant,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Set options")]
pub struct SetOpts {
    /// The version to be set as MSRV
    ///
    /// The given version must be a two- or three component Rust version number.
    /// MSRV values prior to Rust 1.56 will be written to the `package.metadata.msrv` field
    /// in the Cargo manifest. MSRV's greater or equal to 1.56 will be written to
    /// `package.rust-version` in the Cargo manifest.
    #[arg(value_name = "MSRV")]
    pub msrv: BareVersion,

    #[command(flatten)]
    pub rust_releases_opts: RustReleasesOpts,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Verify options")]
pub struct VerifyOpts {
    /// Ignore the lockfile for the MSRV search
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

    #[command(flatten)]
    pub rust_releases_opts: RustReleasesOpts,

    /// The Rust version, to check against for toolchain compatibility
    ///
    /// If not set, the MSRV will be parsed from the Cargo manifest instead.
    #[arg(long, value_name = "rust-version")]
    pub rust_version: Option<BareVersion>,

    #[command(flatten)]
    pub toolchain_opts: ToolchainOpts,

    #[command(flatten)]
    pub custom_check_opts: CustomCheckOpts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CargoCli::command().debug_assert();
    }

    mod top_level {
        use super::*;

        fn assert_find_opts(opts: CargoMsrvOpts, assertions: impl Fn(FindOpts)) {
            if let SubCommand::Find(find_opts) = opts.subcommand {
                assertions(find_opts);
                return;
            }

            panic!("Assertion failed: expected subcommand 'cargo msrv find'");
        }

        mod find_opts {
            use super::*;

            #[test]
            fn has_bisect() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find", "--bisect"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(find_opts.bisect);
                    assert!(!find_opts.linear);
                });
            }

            #[test]
            fn has_not_bisect() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(!find_opts.bisect);
                });
            }

            #[test]
            fn has_linear() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find", "--linear"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(find_opts.linear);
                    assert!(!find_opts.bisect);
                });
            }

            #[test]
            fn has_not_linear() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(!find_opts.linear);
                });
            }

            #[test]
            fn has_write_toolchain_file() {
                let cargo =
                    CargoCli::parse_args(["cargo", "msrv", "find", "--write-toolchain-file"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(find_opts.write_toolchain_file);
                });
            }

            #[test]
            fn has_not_write_toolchain_file() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(!find_opts.write_toolchain_file);
                });
            }

            #[test]
            fn has_ignore_lockfile() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find", "--ignore-lockfile"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(find_opts.ignore_lockfile);
                });
            }

            #[test]
            fn has_not_ignore_lockfile() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(!find_opts.ignore_lockfile);
                });
            }

            #[test]
            fn has_no_check_feedback() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find", "--no-check-feedback"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(find_opts.no_check_feedback);
                });
            }

            #[test]
            fn has_not_no_check_feedback() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(!find_opts.no_check_feedback);
                });
            }

            #[test]
            fn has_write_msrv() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find", "--write-msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(find_opts.write_msrv);
                });
            }

            #[test]
            fn has_not_write_msrv() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "find"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert_find_opts(opts, |find_opts| {
                    assert!(!find_opts.write_msrv);
                });
            }

            // todo: rust-releases opts

            // todo: toolchain opts

            // todo: custom check opts
        }
    }
}
