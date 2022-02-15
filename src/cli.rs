use crate::config::{OutputFormat, TracingTargetOption};
use clap::{App, AppSettings, Arg};
use std::str::FromStr;

use crate::fetch::is_target_available;
use crate::manifest::bare_version::BareVersion;

pub mod id {
    pub const ARG_SEEK_PATH: &str = "seek_path";
    pub const ARG_SEEK_CUSTOM_TARGET: &str = "seek_target";
    pub const ARG_CUSTOM_CHECK: &str = "custom_check";
    pub const ARG_INCLUDE_ALL_PATCH_RELEASES: &str = "include_all_patch";
    pub const ARG_MIN: &str = "min";
    pub const ARG_MAX: &str = "max";
    pub const ARG_BISECT: &str = "bisect";
    pub const ARG_LINEAR: &str = "linear";
    pub const ARG_TOOLCHAIN_FILE: &str = "toolchain_file";
    pub const ARG_IGNORE_LOCKFILE: &str = "lockfile";
    pub const ARG_OUTPUT_FORMAT: &str = "output_format";
    pub const ARG_NO_USER_OUTPUT: &str = "no_user_output";
    pub const ARG_VERIFY: &str = "verify_msrv";
    pub const ARG_RELEASE_SOURCE: &str = "release_source";
    pub const ARG_NO_LOG: &str = "no_log";
    pub const ARG_LOG_LEVEL: &str = "log_level";
    pub const ARG_LOG_TARGET: &str = "log_target";
    pub const ARG_NO_READ_MIN_EDITION: &str = "no_read_min_edition";
    pub const ARG_NO_CHECK_FEEDBACK: &str = "no_check_feedback";

    pub const SUB_COMMAND_LIST: &str = "list";
    pub const SUB_COMMAND_LIST_VARIANT: &str = "list_variant";

    pub const SUB_COMMAND_SET: &str = "set";
    pub const SUB_COMMAND_SET_VALUE: &str = "set_value";

    pub const SUB_COMMAND_SHOW: &str = "show";

    pub const SUB_COMMAND_VERIFY: &str = "verify";
}

pub fn cli() -> App<'static> {
    App::new("cargo-msrv")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Martijn Gribnau <garm@ilumeo.com>")
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::UseLongFormatForHelpSubcommand)
        .global_setting(AppSettings::NextLineHelp)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        // .global_setting(AppSettings::ArgsNegateSubcommands)
        .max_term_width(120)
        .about("Helps with finding the Minimal Supported Rust Version (MSRV)")
        .after_help("\
An argument provided after two dashes (`--`), will be interpreted as a custom command `check` command, \
used to validate whether a Rust toolchain version is compatible. The default `check` command is \
\"cargo build\". A custom `check` command should be runnable by rustup, as they will be passed on to \
rustup like so: `rustup run <toolchain> <COMMAND...>`. You'll only need to provide the <COMMAND...> part.")
        .subcommand(list())
        .subcommand(set())
        .subcommand(show())
        .subcommand(verify())
        .arg(
            Arg::new(id::ARG_SEEK_PATH)
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
            Arg::new(id::ARG_SEEK_CUSTOM_TARGET)
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
        .arg(Arg::new(id::ARG_INCLUDE_ALL_PATCH_RELEASES)
            .long("include-all-patch-releases")
            .help("Include all patch releases, instead of only the last")
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_MIN)
            .long("min")
            .visible_alias("minimum")
            .help("Earliest version to take into account")
            .long_help("Earliest (least recent) version to take into account. \
             Version must match a valid Rust toolchain, and be semver compatible. Edition aliases may also be used.")
            .takes_value(true)
        )
        .arg(Arg::new(id::ARG_MAX)
            .long("max")
            .visible_alias("maximum")
            .help("Latest version to take into account")
            .long_help("Latest (most recent) version to take into account.\
             Version must match a valid Rust toolchain, and be semver compatible.")
            .takes_value(true)
        )
        .arg(Arg::new(id::ARG_BISECT)
            .long("bisect")
            .help("Use a binary search to find the MSRV instead of a linear search")
            .conflicts_with(id::ARG_LINEAR)
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_LINEAR)
            .long("linear")
            .help("Use a linear search to find the MSRV, by checking toolchains from latest to earliest")
            .conflicts_with(id::ARG_BISECT)
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_TOOLCHAIN_FILE)
            .long("toolchain-file")
            .help("Output a rust-toolchain file with the MSRV as toolchain")
            .long_help("Output a rust-toolchain file with the MSRV as toolchain. \
            The toolchain file will pin the Rust version for this crate. \
            See https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file for more.")
        )
        .arg(Arg::new(id::ARG_IGNORE_LOCKFILE)
            .long("ignore-lockfile")
            .help("Temporarily removes the lockfile, so it will not interfere with the building process")
            .long_help("Temporarily removes the lockfile, so it will not interfere with the building process. \
            This is important when testing against Rust versions prior to 1.38.0, for which Cargo does not recognize the new v2 lockfile.")
        )
        .arg(Arg::new(id::ARG_OUTPUT_FORMAT)
            .long("output-format")
            .help("Output status messages in machine-readable format")
            .takes_value(true)
            .possible_values(OutputFormat::custom_formats())
            .long_help("Output status messages in machine-readable format. \
            Machine-readable status updates will be printed in the requested format to stdout. \
            Ignored when used in conjunction with the `--no-user-output` flag.")
        )
        .arg(Arg::new(id::ARG_NO_USER_OUTPUT)
            .long("no-user-output")
            .help("Disables user output")
            .long_help("Disables user output. Useful when when `--log-target stdout` is present, \
            so no clipping between the user output prints and log message prints will take place. \
            When present, the `--output-format [value]` option will be ignored.")
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_VERIFY)
            .long("verify")
            .help("DEPRECATED: Verify the MSRV defined in the 'package.rust-version' or the 'package.metadata.msrv' key in Cargo.toml")
            .long_help("Verify the MSRV defined in the 'package.rust-version' or the 'package.metadata.msrv' key in Cargo.toml. \
            When this flag is present, cargo-msrv will not attempt to determine the true MSRV. \
            Instead it attempts to verify whether for the specified MSRV, the `check` command passes. This is similar to \
            how we determine whether a Rust toolchain version is compatible for your crate or not. \
            DEPRECATED: use the `cargo msrv verify` subcommand instead.")
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_RELEASE_SOURCE)
            .long("release-source")
            .help("Select the rust-releases source to use as the release index")
            .takes_value(true)
            .possible_values(&["rust-changelog", "rust-dist"])
            .default_value("rust-changelog")
        )
        .arg(Arg::new(id::ARG_NO_LOG)
            .long("no-log")
            .help("Disable logging")
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_LOG_TARGET)
            .long("log-target")
            .help("Specify where the program should output its logs")
            .takes_value(true)
            .number_of_values(1)
            .possible_values(&[TracingTargetOption::FILE, TracingTargetOption::STDOUT])
            .default_value(TracingTargetOption::FILE)
        )
        .arg(Arg::new(id::ARG_LOG_LEVEL)
            .long("log-level")
            .help("Specify the verbosity of logs the program should output")
            .takes_value(true)
            .number_of_values(1)
            .possible_values(&["error", "warn", "info", "debug", "trace"])
            .default_value("info")
        )
        .arg(Arg::new(id::ARG_NO_READ_MIN_EDITION)
            .long("no-read-min-edition")
            .help("If provided, the 'package.edition' value in the Cargo.toml will not \
            be used to reduce search space.")
            .takes_value(false)
        )
        .arg(Arg::new(id::ARG_NO_CHECK_FEEDBACK)
            .long("no-check-feedback")
            .help("If provided, the outcome of each individual check will not be printed.")
            .takes_value(false)
        )
        .arg(custom_check())
}

pub fn custom_check() -> Arg<'static> {
    Arg::new(id::ARG_CUSTOM_CHECK)
        .value_name("COMMAND")
        .help(
            "If given, this command is used to validate if a Rust version is \
                compatible. Should be available to rustup, i.e. the command should work like \
                so: `rustup run <toolchain> <COMMAND>`. \
                The default check action is `cargo check`.",
        )
        .multiple_values(true)
        .takes_value(true)
        .required(false)
        .last(true)
}

pub fn list() -> App<'static> {
    use crate::config::list;

    App::new(id::SUB_COMMAND_LIST)
        .about("List the MSRV's specified by dependency crate authors.")
        .arg(
            Arg::new(id::SUB_COMMAND_LIST_VARIANT)
                .long("variant")
                .takes_value(true)
                .possible_values(&[list::DIRECT_DEPS, list::ORDERED_BY_MSRV])
                .default_value(list::ListVariant::default().as_str()),
        )
}

pub fn set() -> App<'static> {
    App::new(id::SUB_COMMAND_SET)
        .arg(
            Arg::new(id::SUB_COMMAND_SET_VALUE)
                .help("The version to be set as MSRV")
                .value_name("MSRV")
                .takes_value(true)
                .required(true)
                .validator(BareVersion::from_str),
        )
        .about("Set the MSRV of the current crate to a given Rust version.")
}

pub fn show() -> App<'static> {
    App::new(id::SUB_COMMAND_SHOW)
        .about("Show the MSRV of your crate, as specified in the Cargo manifest.")
        .after_help(
            "The given version must be a two- or three component Rust version number. \
                   MSRV values prior to Rust 1.56 will be written to the `package.metadata.msrv` field \
                   in the Cargo manifest, \
                   while MSRV's greater or equal to 1.56 will be written to `package.rust-version` \
                   in the Cargo manifest.",
        )
}

pub fn verify() -> App<'static> {
    App::new(id::SUB_COMMAND_VERIFY)
        .about("Verify whether the MSRV is satisfiable. The MSRV must be specified using the 'package.rust-version' or 'package.metadata.msrv' key in the Cargo.toml manifest.")
        .arg(custom_check())
}

#[cfg(test)]
mod tests {
    use crate::cli::cli;

    #[test]
    fn cli_conforms() {
        cli().debug_assert();
    }
}
