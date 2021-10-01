use crate::fetch::is_target_available;
use clap::{App, AppSettings, Arg, SubCommand};

pub mod id {
    pub const SUB_COMMAND_MSRV: &str = "msrv";
    pub const ARG_SEEK_PATH: &str = "seek_path";
    pub const ARG_SEEK_CUSTOM_TARGET: &str = "seek_target";
    pub const ARG_CUSTOM_CHECK: &str = "custom_check";
    pub const ARG_INCLUDE_ALL_PATCH_RELEASES: &str = "include_all_patch";
    pub const ARG_MIN: &str = "min";
    pub const ARG_MAX: &str = "max";
    pub const ARG_BISECT: &str = "bisect";
    pub const ARG_TOOLCHAIN_FILE: &str = "toolchain_file";
    pub const ARG_IGNORE_LOCKFILE: &str = "lockfile";
    pub const ARG_OUTPUT_FORMAT: &str = "output_format";
    pub const ARG_VERIFY: &str = "verify_msrv";
}

pub fn cli() -> App<'static, 'static> {
    App::new("cargo-msrv")
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Martijn Gribnau <garm@ilumeo.com>")
        .global_setting(AppSettings::NextLineHelp)
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .max_term_width(120)
        .subcommand(
            SubCommand::with_name(id::SUB_COMMAND_MSRV)
                .usage("cargo msrv [OPTIONS]")
                .about("Helps with finding the Minimal Supported Rust Version (MSRV)")
                .after_help("\
An argument provided after two dashes (`--`), will be interpreted as a custom command `check` command, \
used to validate whether a Rust toolchain version is compatible. The default `check` command is \
\"cargo build\". A custom `check` command should be runnable by rustup, as they will be passed on to \
rustup like so: `rustup run <toolchain> <COMMAND...>`. You'll only need to provide the <COMMAND...> part.")
                .arg(
                    Arg::with_name(id::ARG_SEEK_PATH)
                        .long("path")
                        .help("Path to the cargo project directory")
                        .takes_value(true)
                        .value_name("DIR")
                        .validator(|value| {
                            std::fs::metadata(&value)
                                .map_err(|_| "Path doesn't exist.".to_string())
                                .and_then(|m| {
                                    if m.is_dir() {
                                        Ok(())
                                    } else {
                                        Err("Not a directory.".to_string())
                                    }
                                })
                        }),
                )
                .arg(
                    Arg::with_name(id::ARG_SEEK_CUSTOM_TARGET)
                        .long("target")
                        .help("Check against a custom target (instead of the rustup default)")
                        .takes_value(true)
                        .value_name("TARGET")
                        .validator(|value| {
                            is_target_available(&value).map_err(|_| {
                                "The provided target is not available. Use `rustup target list` to \
                                 review the available targets."
                                    .to_string()
                            })
                        }),
                )
                .arg(Arg::with_name(id::ARG_INCLUDE_ALL_PATCH_RELEASES)
                    .long("include-all-patch-releases")
                    .help("Include all patch releases, instead of only the last")
                    .takes_value(false)
                )
                .arg(Arg::with_name(id::ARG_MIN)
                    .long("min")
                    .visible_alias("minimum")
                    .help("Earliest version to take into account")
                    .long_help("Earliest (least recent) version to take into account. \
                     Version must match a valid Rust toolchain, and be semver compatible. Edition aliases may also be used.")
                    .takes_value(true)
                )
                .arg(Arg::with_name(id::ARG_MAX)
                    .long("max")
                    .visible_alias("maximum")
                    .help("Latest version to take into account")
                    .long_help("Latest (most recent) version to take into account.\
                     Version must match a valid Rust toolchain, and be semver compatible.")
                    .takes_value(true)
                )
                .arg(Arg::with_name(id::ARG_BISECT)
                    .long("bisect")
                    .help("Use a binary search to find the MSRV instead of a linear search")
                    .takes_value(false)
                )
                .arg(Arg::with_name(id::ARG_TOOLCHAIN_FILE)
                    .long("toolchain-file")
                    .help("Output a rust-toolchain file with the MSRV as toolchain")
                    .long_help("Output a rust-toolchain file with the MSRV as toolchain. \
                    The toolchain file will pin the Rust version for this crate. \
                    See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more.")
                )
                .arg(Arg::with_name(id::ARG_IGNORE_LOCKFILE)
                    .long("ignore-lockfile")
                    .help("Temporarily removes the lockfile, so it will not interfere with the building process")
                    .long_help("Temporarily removes the lockfile, so it will not interfere with the building process. \
                    This is important when testing against Rust versions prior to 1.38.0, for which Cargo does not recognize the new v2 lockfile.")
                )
                .arg(Arg::with_name(id::ARG_OUTPUT_FORMAT)
                    .long("output-format")
                    .help("Output status messages in machine-readable format")
                    .takes_value(true)
                    .possible_values(&["json"])
                    .long_help("Output status messages in machine-readable format. \
                Machine-readable status updates will be printed in the requested format to stdout.")
                )
                .arg(Arg::with_name(id::ARG_VERIFY)
                    .long("verify")
                    .help("Verify the MSRV defined in the 'package.metadata.msrv' key in Cargo.toml")
                    .long_help("Verify the MSRV defined in the 'package.metadata.msrv' key in Cargo.toml. \
                    When this flag is present, cargo-msrv will not attempt to determine the true MSRV. \
                    Instead it attempts to verify whether for the specified MSRV, the `check` command passes. This is similar to \
                    how we determine whether a Rust toolchain version is compatible for your crate or not.")
                    .takes_value(false)
                )
                .arg(
                    Arg::with_name(id::ARG_CUSTOM_CHECK)
                        .value_name("COMMAND")
                        .help("If given, this command is used to validate if a Rust version is \
                        compatible. Should be available to rustup, i.e. the command should work like \
                        so: `rustup run <toolchain> <COMMAND>`. \
                        The default check action is `cargo check --all`.")
                        .multiple(true)
                        .last(true)

                )
        )
}
