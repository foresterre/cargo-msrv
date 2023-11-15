use crate::cli::custom_check_opts::CheckCommandOpts;
use crate::cli::find_opts::FindOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use crate::context::list::ListMsrvVariant;
use crate::manifest::bare_version::BareVersion;
use clap::{Args, Parser, Subcommand};
use std::ffi::{OsStr, OsString};

pub(crate) mod custom_check_opts;
pub(crate) mod find_opts;
pub(crate) mod rust_releases_opts;
pub(crate) mod shared_opts;
pub(crate) mod toolchain_opts;

#[derive(Debug, Parser)]
#[command(version, name = "cargo", bin_name = "cargo", max_term_width = 120)]
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
            You may provide a custom compatibility `check` command as the last argument (only
            when this argument is provided via the double dash syntax, e.g. `$ cargo msrv -- custom
            command`.
            This custom check command will then be used to validate whether a Rust version is
            compatible.
            A custom `check` command should be runnable by rustup, as they will be passed on to
            rustup like so: `rustup run <toolchain> <COMMAND...>`. NB: You only need to provide the
            <COMMAND...> part.

            By default, the custom check command is `cargo check`.
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
    pub find_opts: FindOpts,

    #[command(flatten)]
    pub shared_opts: SharedOpts,

    #[command(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
#[command(propagate_version = true)]
pub enum SubCommand {
    /// Display the MSRV's of dependencies
    List(ListOpts),
    /// Set the MSRV of the current crate to a given Rust version
    Set(SetOpts),
    /// Show the MSRV of your crate, as specified in the Cargo manifest
    Show,
    /// Verify whether the MSRV is satisfiable. The MSRV must be specified using the
    /// 'package.rust-version' or 'package.metadata.msrv' key in the Cargo.toml manifest.
    Verify(VerifyOpts),
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
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Verify options")]
pub struct VerifyOpts {
    #[command(flatten)]
    pub rust_releases_opts: RustReleasesOpts,

    #[command(flatten)]
    pub toolchain_opts: ToolchainOpts,

    #[command(flatten)]
    pub cargo_check_opts: CheckCommandOpts,

    /// The Rust version, to check against for toolchain compatibility
    ///
    /// If not set, the MSRV will be parsed from the Cargo manifest instead.
    #[arg(long, value_name = "rust-version")]
    pub rust_version: Option<BareVersion>,
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

        mod find_opts {
            use super::*;

            #[test]
            fn has_bisect() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "--bisect"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(opts.find_opts.bisect);
                assert!(!opts.find_opts.linear);
            }

            #[test]
            fn has_not_bisect() {
                let cargo = CargoCli::parse_args(["cargo", "msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(!opts.find_opts.bisect);
            }

            #[test]
            fn has_linear() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "--linear"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(opts.find_opts.linear);
                assert!(!opts.find_opts.bisect);
            }

            #[test]
            fn has_not_linear() {
                let cargo = CargoCli::parse_args(["cargo", "msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(!opts.find_opts.linear);
            }

            #[test]
            fn has_write_toolchain_file() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "--write-toolchain-file"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(opts.find_opts.write_toolchain_file);
            }

            #[test]
            fn has_not_write_toolchain_file() {
                let cargo = CargoCli::parse_args(["cargo", "msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(!opts.find_opts.write_toolchain_file);
            }

            #[test]
            fn has_ignore_lockfile() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "--ignore-lockfile"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(opts.find_opts.ignore_lockfile);
            }

            #[test]
            fn has_not_ignore_lockfile() {
                let cargo = CargoCli::parse_args(["cargo", "msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(!opts.find_opts.ignore_lockfile);
            }

            #[test]
            fn has_no_check_feedback() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "--no-check-feedback"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(opts.find_opts.no_check_feedback);
            }

            #[test]
            fn has_not_no_check_feedback() {
                let cargo = CargoCli::parse_args(["cargo", "msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(!opts.find_opts.no_check_feedback);
            }

            #[test]
            fn has_write_msrv() {
                let cargo = CargoCli::parse_args(["cargo", "msrv", "--write-msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(opts.find_opts.write_msrv);
            }

            #[test]
            fn has_not_write_msrv() {
                let cargo = CargoCli::parse_args(["cargo", "msrv"]);
                let cargo_msrv = cargo.to_cargo_msrv_cli();
                let opts = cargo_msrv.to_opts();

                assert!(!opts.find_opts.write_msrv);
            }

            // todo: rust-releases opts

            // todo: toolchain opts

            // todo: custom check opts
        }
    }
}
